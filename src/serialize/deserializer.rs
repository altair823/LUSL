use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use crate::{
    binary::verify_checksum,
    compress::{decompress, TEMP_COMPRESSED_FILE_PATH},
    encrypt::{make_decryptor, make_key_from_password_and_salt, NONCE_LENGTH, SALT_LENGTH},
};

use super::{header::FILE_LABEL, meta::MetaData, BUFFER_LENGTH};
use super::{
    header::{get_major_version, get_minor_version, Header},
    option::SerializeOption,
};

/// # Deserializer
///
/// Deserializer struct.
///
/// Call deserialize method for deserialize data file.
///
/// Checking [MD5](md5) checksum of files and if it is different, occur error.
///
/// # Examples
/// ```
/// use lusl::{Serializer, Deserializer, SerializeOption};
/// use std::path::PathBuf;
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized2.bin");
/// let mut serializer = Serializer::new(&original, &result).unwrap();
/// serializer.serialize().unwrap();
///
/// let restored = PathBuf::from("deserialized_dir");
/// let mut deserializer = Deserializer::new(&result, &restored).unwrap();
/// deserializer.deserialize().unwrap();
/// assert!(&result.is_file());
/// assert!(&restored.is_dir());
/// ```
pub struct Deserializer {
    serialized_file: BufReader<File>,
    buffer: VecDeque<u8>,
    restore_path: PathBuf,
    option: SerializeOption,
    sender: Option<Sender<String>>,
}

impl Deserializer {
    /// Set serialized data file path and restored file path.
    pub fn new<T: AsRef<Path>>(serialized_file: T, restore_path: T) -> io::Result<Self> {
        let serialized_file_path = serialized_file.as_ref().to_path_buf();
        if let false = serialized_file_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File doesn't exists!",
            ));
        }
        Ok(Deserializer {
            serialized_file: BufReader::with_capacity(
                BUFFER_LENGTH,
                File::open(serialized_file_path)?,
            ),
            buffer: VecDeque::with_capacity(BUFFER_LENGTH + 16),
            restore_path: restore_path.as_ref().to_path_buf(),
            option: SerializeOption::default(),
            sender: None,
        })
    }

    /// Set option for deserializer.
    pub fn set_option(&mut self, option: SerializeOption) {
        self.option = option;
    }

    /// Set transmitter to send progress.
    /// If you don't want to send progress, don't call this method.
    pub fn set_sender(&mut self, tx: Sender<String>) {
        self.sender = Some(tx);
    }

    fn fill_buf(&mut self) -> io::Result<usize> {
        self.buffer.append(&mut VecDeque::from_iter(
            self.serialized_file.fill_buf()?.to_vec(),
        ));
        self.serialized_file.consume(self.buffer.len());
        Ok(self.buffer.len())
    }

    fn fill_buf_with_len(&mut self, length: usize) -> io::Result<Vec<u8>> {
        while self.buffer.len() < length {
            let previous_buf_len = self.buffer.len();
            self.fill_buf()?;
            if self.buffer.len() == previous_buf_len {
                return Ok(self.buffer.drain(..self.buffer.len()).collect());
            }
        }
        Ok(self.buffer.drain(..length).collect())
    }
    /// Deserialize data file to directory.
    ///
    /// If the file encrypted, deserializing with given password which is in the option.
    ///
    /// After deserializing a file is completed, checking [MD5](md5) checksum of files and if it is different, occur error.
    ///
    /// # Errors
    /// - Wrong file format or data.
    /// - MD5 checksum of deserialized file is different from original checksum.
    /// - Wrong password.
    pub fn deserialize(&mut self) -> io::Result<()> {
        let header = self.verify_header()?;
        let original_file_count = header.file_count();
        match header.is_encrypted() {
            true => self.deserialize_with_decrypt(
                &match self.option.password() {
                    Some(p) => p,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "This file is encrypted but there is no password input.",
                        ))
                    }
                },
                original_file_count,
            )?,
            false => self.deserialize_raw(original_file_count)?,
        }
        Ok(())
    }

    fn send_progress(&self, message: &str) {
        if let Some(ref tx) = self.sender {
            tx.send(message.to_string()).unwrap();
        }
    }

    fn deserialize_raw(&mut self, original_file_count: u64) -> io::Result<()> {
        let mut current_file_count: u64 = 0;
        loop {
            let metadata = self.read_metadata()?;

            // Write file
            let file_path = self.restore_path.join(&metadata.path());
            fs::create_dir_all(self.restore_path.join(&metadata.path()).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&metadata.path()))?;
            match self.option.is_compressed() {
                true => {
                    let mut compressed_size = 0u64;
                    let t = self.fill_buf_with_len(8)?;
                    compressed_size += t[0] as u64 * 0x1;
                    compressed_size += t[1] as u64 * 0x100;
                    compressed_size += t[2] as u64 * 0x10000;
                    compressed_size += t[3] as u64 * 0x1000000;
                    let temp_file = PathBuf::from(TEMP_COMPRESSED_FILE_PATH)
                        .join(metadata.path().file_name().unwrap());
                    self.write_raw_file(&temp_file, compressed_size as usize)?;
                    let a = decompress(&temp_file, TEMP_COMPRESSED_FILE_PATH)?;
                    fs::rename(a, &file_path)?;
                }
                false => {
                    self.write_raw_file(&file_path, metadata.size() as usize)?;
                }
            }

            // Verify checksum
            verify_checksum(metadata, &file_path)?;

            // Count file.
            current_file_count += 1;

            // Send progress.
            self.send_progress(&format!(
                "Deserializing... {} / {}    {}",
                current_file_count,
                original_file_count,
                &file_path.to_str().unwrap()
            ));

            // EOF.
            if self.buffer.len() == 0 {
                if self.fill_buf()? == 0 {
                    break;
                } else {
                    continue;
                }
            }
        }
        if original_file_count != current_file_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Number of files is different with the original directory!",
            ));
        }
        if PathBuf::from(TEMP_COMPRESSED_FILE_PATH).is_dir() {
            fs::remove_dir_all(TEMP_COMPRESSED_FILE_PATH)?;
        }
        Ok(())
    }

    fn deserialize_with_decrypt(
        &mut self,
        password: &str,
        original_file_count: u64,
    ) -> io::Result<()> {
        let mut current_file_count: u64 = 0;
        // Read salt and key.
        let salt = self.fill_buf_with_len(SALT_LENGTH)?;
        let key = make_key_from_password_and_salt(password, salt);

        loop {
            let metadata = self.read_metadata()?;

            // Write file
            let file_path = self.restore_path.join(&metadata.path());
            fs::create_dir_all(self.restore_path.join(&metadata.path()).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&metadata.path()))?;
            match self.option.is_compressed() {
                true => {
                    let mut compressed_size = 0u64;
                    let t = self.fill_buf_with_len(8)?;
                    compressed_size += t[0] as u64 * 0x1;
                    compressed_size += t[1] as u64 * 0x100;
                    compressed_size += t[2] as u64 * 0x10000;
                    compressed_size += t[3] as u64 * 0x1000000;
                    let temp_file = PathBuf::from(TEMP_COMPRESSED_FILE_PATH)
                        .join(metadata.path().file_name().unwrap());
                    self.write_decrypt_file(&temp_file, compressed_size as usize, &key)?;
                    let a = decompress(&temp_file, TEMP_COMPRESSED_FILE_PATH)?;
                    fs::rename(a, &file_path)?;
                }
                false => {
                    self.write_decrypt_file(&file_path, metadata.size() as usize, &key)?;
                }
            }

            // Verify checksum
            verify_checksum(metadata, &file_path)?;

            // Count file.
            current_file_count += 1;

            // Send progress.
            self.send_progress(&format!(
                "Deserializing... {} / {}    {}",
                current_file_count,
                original_file_count,
                &file_path.to_str().unwrap()
            ));

            // EOF.
            if self.buffer.len() == 0 {
                if self.fill_buf()? == 0 {
                    break;
                } else {
                    continue;
                }
            }
        }
        if original_file_count != current_file_count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Number of files is different with the original directory!",
            ));
        }
        if PathBuf::from(TEMP_COMPRESSED_FILE_PATH).is_dir() {
            fs::remove_dir_all(TEMP_COMPRESSED_FILE_PATH)?;
        }
        Ok(())
    }

    fn verify_header(&mut self) -> io::Result<Header> {
        // Verify label.
        let mut header = Header::new();
        header.deserialize_label(&self.fill_buf_with_len(FILE_LABEL.as_bytes().len())?)?;

        // Verify version.
        header.deserialize_version(&self.fill_buf_with_len(4)?)?;
        if header.version().major() < get_major_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The major version of the file is too low. It is a serialized file with an older version of the library. \
                To deserialize this file, library version {}.x.x is required. \
                If you want to deserialize this file, Use an older version of the library.", header.version().major()),
            ));
        } else if header.version().major() > get_major_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The major version of the file is too high. It is a serialized file with a newer version of the library. \
                To deserialize this file, library version {}.{}.x is required. \
                If you want to deserialize this file, Use a newer version of the library.", header.version().major(), header.version().minor()),
            ));
        } else if header.version().minor() > get_minor_version() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("The minor version of the file is too high. \
                It is a serialized file with a newer version of the library. \
                To deserialize this file, library version {}.{}.x is required. If you want to deserialize this file, Use a newer version of the library.", header.version().major(), header.version().minor()),
            ));
        }

        // Read header flags.
        header.deserialize_flag(&self.fill_buf_with_len(1)?);

        // Verify header flags.
        match header.is_compressed() {
            true => {
                if !self.option.is_compressed() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "The archive is compressed, but the option is not set.",
                    ));
                }
            }
            false => {
                if self.option.is_compressed() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "The archive is not compressed, but the option is set.",
                    ));
                }
            }
        }
        match header.is_encrypted() {
            true => {
                if !self.option.is_encrypted() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "The archive is encrypted, but the option is not set.",
                    ));
                }
            }
            false => {
                if self.option.is_encrypted() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "The archive is not encrypted, but the option is set.",
                    ));
                }
            }
        }

        // Read the number of original files.
        let original_file_count_bytes = self.fill_buf_with_len(1)?[0];
        header.deserialize_file_count(&self.fill_buf_with_len(original_file_count_bytes as usize)?);

        Ok(header)
    }

    fn read_metadata(&mut self) -> io::Result<MetaData> {
        let mut metadata = MetaData::new();

        // Restore file path
        let path_size_bin = self.fill_buf_with_len(2)?;
        let path_size = path_size_bin[0] as usize * 0x100 + path_size_bin[1] as usize;
        metadata.deserialize_path(&self.fill_buf_with_len(path_size)?);

        // Restore file type
        let flag_and_byte_count = self.fill_buf_with_len(1)?[0];
        metadata.deserialize_type(flag_and_byte_count);

        // Restore file size
        let size_count = (flag_and_byte_count & 0xF) as usize;
        metadata.deserialize_size(&self.fill_buf_with_len(size_count)?);

        // Restore checksum
        metadata.deserialize_checksum(&self.fill_buf_with_len(32)?);

        Ok(metadata)
    }

    fn write_raw_file<T: AsRef<Path>>(
        &mut self,
        restored_file_path: T,
        size: usize,
    ) -> io::Result<()> {
        match restored_file_path.as_ref().parent() {
            Some(p) => fs::create_dir_all(p)?,
            None => (),
        }
        let mut file = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&restored_file_path)?,
        );
        let mut counter = 0;
        loop {
            counter += self.fill_buf()?;
            if counter > size {
                if size > self.buffer.len() {
                    file.write(
                        &Vec::from(self.buffer.clone())[..self.buffer.len() - (counter - size)],
                    )?;
                    self.buffer.drain(..self.buffer.len() - (counter - size));
                } else {
                    file.write(&Vec::from(self.buffer.clone())[..size])?;
                    self.buffer.drain(..size);
                }
                file.flush()?;
                break;
            }

            file.write(&Vec::from(self.buffer.clone()))?;
            self.buffer.clear();
            if counter == size {
                file.flush()?;
                break;
            }
        }
        Ok(())
    }

    fn write_decrypt_file<T: AsRef<Path>>(
        &mut self,
        restored_file_path: T,
        mut size: usize,
        key: &[u8],
    ) -> io::Result<()> {
        match restored_file_path.as_ref().parent() {
            Some(p) => fs::create_dir_all(p)?,
            None => (),
        }
        let mut file = BufWriter::with_capacity(
            BUFFER_LENGTH + 16,
            OpenOptions::new()
                .create(true)
                .write(true)
                .open(&restored_file_path)?,
        );
        let nonce = self.fill_buf_with_len(NONCE_LENGTH)?;
        let mut decryptor = make_decryptor(key, &nonce);
        let mut counter = 0;
        loop {
            let mut temp = self.fill_buf_with_len(BUFFER_LENGTH + 16)?;
            size += 16;
            counter += temp.len();
            if counter > size {
                let decrypted_data = decryptor
                    .decrypt_last(&temp[..BUFFER_LENGTH + 16 - (counter - size)])
                    .expect("decrypt failed");
                file.write(&decrypted_data)?;
                let mut new_buf = VecDeque::new();
                new_buf.extend(&temp[BUFFER_LENGTH + 16 - (counter - size)..]);
                new_buf.append(&mut self.buffer);
                self.buffer = new_buf;
                file.flush()?;
                break;
            }

            if counter == size {
                if temp.len() == BUFFER_LENGTH + 16 {
                    let decrypted_data = decryptor
                        .decrypt_next(temp.as_slice())
                        .expect("decrypt failed");
                    file.write(&decrypted_data)?;
                    let temp = self.fill_buf_with_len(16)?;
                    let decrypted_data = decryptor
                        .decrypt_last(temp.as_slice())
                        .expect("decrypt failed");
                    file.write(&decrypted_data)?;
                } else {
                    let decrypted_data = decryptor
                        .decrypt_last(temp.as_slice())
                        .expect("decrypt failed");
                    file.write(&decrypted_data)?;
                }
                file.flush()?;

                break;
            }
            let decrypted_data = decryptor
                .decrypt_next(temp.as_slice())
                .expect("decrypt failed");
            file.write(&decrypted_data)?;
            temp.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::serialize::serializer::Serializer;

    use super::*;
    use std::{path::PathBuf, sync::mpsc, thread};

    #[test]
    fn deserialize_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("deserialize_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(SerializeOption::default());
        serializer.serialize().unwrap();

        let serialized_file = PathBuf::from("deserialize_test.bin");
        let restored = PathBuf::from("deserialize_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer.set_option(SerializeOption::default());
        deserializer.deserialize().unwrap();
        assert!(&result.is_file());
        assert!(&restored.is_dir());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }

    #[test]
    fn deserialize_with_decrypt_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("deserialize_with_decrypt_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(SerializeOption::new().to_encrypt("test_password"));
        serializer.serialize().unwrap();

        let serialized_file = PathBuf::from("deserialize_with_decrypt_test.bin");
        let restored = PathBuf::from("deserialize_with_decrypt_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer.set_option(SerializeOption::new().to_encrypt("test_password"));
        deserializer.deserialize().unwrap();
        assert!(&result.is_file());
        assert!(&restored.is_dir());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }

    #[test]
    fn deserialize_with_compress_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("deserialize_compress_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(SerializeOption::new().to_compress(true));
        serializer.serialize().unwrap();

        let serialized_file = PathBuf::from("deserialize_compress_test.bin");
        let restored = PathBuf::from("deserialize_compress_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer.set_option(SerializeOption::new().to_compress(true));
        deserializer.deserialize().unwrap();
        assert!(&result.is_file());
        assert!(&restored.is_dir());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }

    #[test]
    fn deserialize_with_decrypt_compress_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("deserialize_decrypt_compress_test.bin");
        let option = SerializeOption::new()
            .to_compress(true)
            .to_encrypt("test_password");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(option.clone());
        serializer.serialize().unwrap();

        let serialized_file = PathBuf::from("deserialize_decrypt_compress_test.bin");
        let restored = PathBuf::from("deserialize_decrypt_compress_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer.set_option(option.clone());
        deserializer.deserialize().unwrap();
        assert!(&result.is_file());
        assert!(&restored.is_dir());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }

    #[test]
    fn deserialize_sender_test() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let original = PathBuf::from("tests");
            let result = PathBuf::from("deserialize_sender_test.bin");
            let mut serializer = Serializer::new(original, result.clone()).unwrap();
            serializer.set_option(SerializeOption::default());
            serializer.serialize().unwrap();

            let serialized_file = PathBuf::from("deserialize_sender_test.bin");
            let restored = PathBuf::from("deserialize_sender_test_dir");
            let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
            deserializer.set_option(SerializeOption::default());
            deserializer.set_sender(tx);
            deserializer.deserialize().unwrap();
            assert!(&result.is_file());
            assert!(&restored.is_dir());
            if result.is_file() {
                fs::remove_file(result).unwrap();
            }
            if restored.is_dir() {
                fs::remove_dir_all(restored).unwrap();
            }
        });
        let mut msgs = Vec::new();
        for msg in rx {
            msgs.push(msg);
        }
        assert_eq!(msgs, ["Deserializing... 1 / 10    deserialize_sender_test_dir/tests/original_images/dir1/laboratory-g8f9267f5f_1920.jpg", 
        "Deserializing... 2 / 10    deserialize_sender_test_dir/tests/original_images/dir1/board-g43968feec_1920.jpg", 
        "Deserializing... 3 / 10    deserialize_sender_test_dir/tests/original_images/dir1/폭발.jpg", 
        "Deserializing... 4 / 10    deserialize_sender_test_dir/tests/original_images/dir2/capsules-g869437822_1920.jpg", 
        "Deserializing... 5 / 10    deserialize_sender_test_dir/tests/original_images/dir4/colorful-2174045.png", 
        "Deserializing... 6 / 10    deserialize_sender_test_dir/tests/original_images/dir2/dir3/syringe-ge5e95bfe6_1920.jpg", 
        "Deserializing... 7 / 10    deserialize_sender_test_dir/tests/original_images/dir2/dir3/books-g6617d4d97_1920.jpg", 
        "Deserializing... 8 / 10    deserialize_sender_test_dir/tests/original_images/dir4/dir5/digitization-1755812_1920.jpg", 
        "Deserializing... 9 / 10    deserialize_sender_test_dir/tests/original_images/dir4/dir5/dir6/tv-g87676cdfb_1280.png",
        "Deserializing... 10 / 10    deserialize_sender_test_dir/tests/original_images/dir4/dir5/dir6/test-pattern-152459.png"]);
    }
}
