use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{
    binary::verify_checksum,
    encrypt::{make_decryptor, make_key_from_password_and_salt, NONCE_LENGTH, SALT_LENGTH},
};

use super::{header::Header, option::SerializeOption};
use super::{header::FILE_LABEL, meta::MetaData, BUFFER_LENGTH};

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
/// let mut serializer = Serializer::new(original, result.clone()).unwrap();
/// serializer.serialize(&SerializeOption::default()).unwrap();
/// let serialized_file = PathBuf::from("serialized2.bin");
/// let restored = PathBuf::from("deserialized_dir");
/// let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
/// deserializer.deserialize(&SerializeOption::default()).unwrap();
/// assert!(&result.is_file());
/// assert!(&restored.is_dir());
/// ```
pub struct Deserializer {
    serialized_file: BufReader<File>,
    buffer: VecDeque<u8>,
    restore_path: PathBuf,
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
        })
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
    pub fn deserialize(&mut self, option: &SerializeOption) -> io::Result<()> {
        let header = self.read_header()?;
        let original_file_count = header.file_count();
        match header.is_encrypted() {
            true => self.deserialize_with_decrypt(
                &match option.password() {
                    Some(p) => p,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "No password input.",
                        ))
                    }
                },
                original_file_count,
            )?,
            false => self.deserialize_raw(original_file_count)?,
        }
        Ok(())
    }

    fn deserialize_raw(&mut self, original_file_count: u64) -> io::Result<()> {
        let mut current_file_count: u64 = 0;
        loop {
            let metadata = self.read_metadata()?;

            // Write file
            let file_path = self.restore_path.join(&metadata.path());
            fs::create_dir_all(self.restore_path.join(&metadata.path()).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&metadata.path()))?;
            self.write_raw_file(&file_path, metadata.size() as usize)?;

            // Verify checksum
            verify_checksum(metadata, file_path)?;

            // Count file.
            current_file_count += 1;

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
            self.write_decrypt_file(&file_path, metadata.size() as usize, &key)?;

            // Verify checksum
            verify_checksum(metadata, file_path)?;

            // Count file.
            current_file_count += 1;

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

        Ok(())
    }

    fn read_header(&mut self) -> io::Result<Header> {
        // Verify label.
        let mut header = Header::new();
        header.deserialize_label(&self.fill_buf_with_len(FILE_LABEL.as_bytes().len())?)?;

        // Read header flags.
        header.deserialize_flag(&self.fill_buf_with_len(1)?);

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
        let mut file = BufWriter::new(
            OpenOptions::new()
                .append(true)
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
        let mut file = BufWriter::with_capacity(
            BUFFER_LENGTH + 16,
            OpenOptions::new()
                .append(true)
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
                if size > temp.len() {
                    let decrypted_data = decryptor
                        .decrypt_last(&temp[..BUFFER_LENGTH + 16 - (counter - size)])
                        .unwrap();
                    file.write(&decrypted_data.clone())?;
                    let mut new_buf = VecDeque::new();
                    new_buf.extend(&temp[BUFFER_LENGTH + 16 - (counter - size)..]);
                    new_buf.append(&mut self.buffer);
                    self.buffer = new_buf;
                } else {
                    let decrypted_data = decryptor.decrypt_last(&temp[..size]).unwrap();
                    file.write(&decrypted_data)?;
                    let mut new_buf = VecDeque::new();
                    new_buf.extend(&temp[size..]);
                    new_buf.append(&mut self.buffer);
                    self.buffer = new_buf;
                }
                file.flush()?;
                break;
            }

            if counter == size {
                let decrypted_data = decryptor.decrypt_last(temp.as_slice()).unwrap();
                file.write(&decrypted_data)?;
                file.flush()?;
                break;
            }
            let decrypted_data = decryptor.decrypt_next(temp.as_slice()).unwrap();
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
    use std::path::PathBuf;

    #[test]
    fn deserialize_file_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("deserialize_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.serialize(&SerializeOption::default()).unwrap();

        let serialized_file = PathBuf::from("deserialize_test.bin");
        let restored = PathBuf::from("deserialize_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer
            .deserialize(&SerializeOption::default())
            .unwrap();
        assert!(&result.is_file());
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
        serializer
            .serialize(&SerializeOption::new().to_encrypt("test_password"))
            .unwrap();

        let serialized_file = PathBuf::from("deserialize_with_decrypt_test.bin");
        let restored = PathBuf::from("deserialize_with_decrypt_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer
            .deserialize(&SerializeOption::new().to_encrypt("test_password"))
            .unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }

    fn t() {
        let original = PathBuf::from("/mnt/c/Users/rlaxo/Desktop/실록_compressed");
        let result = PathBuf::from("/mnt/c/Users/rlaxo/Desktop/deserialize_with_decrypt_test.bin");
        let option = SerializeOption::new().to_encrypt("823eric!@");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.serialize(&option).unwrap();

        let serialized_file =
            PathBuf::from("/mnt/c/Users/rlaxo/Desktop/deserialize_with_decrypt_test.bin");
        let restored =
            PathBuf::from("/mnt/c/Users/rlaxo/Desktop/deserialize_with_decrypt_test_dir");
        let mut deserializer = Deserializer::new(serialized_file, restored.clone()).unwrap();
        deserializer.deserialize(&option).unwrap();
    }
}
