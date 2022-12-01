use crate::{
    compress::{self, TEMP_COMPRESSED_FILE_PATH},
    encrypt::{make_encryptor, make_new_key_from_password, make_nonce},
};

use super::{
    get_file_list, header::Header, meta::MetaData, option::SerializeOption, BUFFER_LENGTH,
};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

///
/// # Serializer
///
/// Serializer struct.
///
/// Call `serialize` method to serialize all directory contents.   
///
/// ## Usages
/// 
/// ```rust
/// use lusl::{Serializer, SerializeOption};
/// use std::path::PathBuf;
/// use std::fs;
///
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized1.bin");
/// let mut serializer = Serializer::new(&original, &result).unwrap();
/// serializer.serialize().unwrap();
/// assert!(result.is_file());
/// ```

pub struct Serializer {
    parent: PathBuf,
    original_file_list: Vec<PathBuf>,
    result: BufWriter<File>,
    option: SerializeOption,
}

impl Serializer {
    /// Set original root directory and result path and create Serializer.
    /// May create result file.
    pub fn new<T: AsRef<Path>>(original_root: T, result_path: T) -> io::Result<Self> {
        let result_path = PathBuf::from(result_path.as_ref());
        if result_path.is_file() {
            match fs::remove_file(&result_path) {
                Ok(_) => (),
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "File already exists!",
                    ))
                }
            }
        }
        File::create(&result_path)?;
        Ok(Serializer {
            parent: original_root.as_ref().parent().unwrap().to_path_buf(),
            original_file_list: get_file_list(original_root)?,
            result: BufWriter::new(
                OpenOptions::new()
                    .append(true)
                    .write(true)
                    .open(result_path)?,
            ),
            option: SerializeOption::default(),
        })
    }

    /// Set option to serialize.
    pub fn set_option(&mut self, option: SerializeOption) {
        self.option = option;
    }

    /// Serialize root directory and copy it to result file.
    /// 
    /// If `option.compress` is true, compress result file.
    /// 
    /// If `option.encrypt` is true, encrypt result file.
    pub fn serialize(&mut self) -> io::Result<()> {
        match self.option.is_encrypted() {
            true => self.serialize_with_encrypt(&self.option.password().unwrap())?,
            false => self.serialize_raw()?,
        }
        Ok(())
    }

    fn serialize_raw(&mut self) -> io::Result<()> {
        let header = Header::with(false, false, self.original_file_list.len() as u64);
        self.result.write(&header.to_binary_vec())?;
        for i in 0..self.original_file_list.len() {
            // Write metadata.
            let mut metadata = MetaData::from(&self.original_file_list[i]);
            metadata.strip_prefix(&self.parent);
            self.result.write(&metadata.serialize())?;

            // Write binary data.
            let original_file = self.original_file_list[i].clone();
            match self.option.is_compressed() {
                true => {
                    let compressed_file =
                        compress::compress(original_file, TEMP_COMPRESSED_FILE_PATH)?;
                    self.result
                        .write(&compressed_file.metadata()?.len().to_le_bytes().to_vec())?;
                    self.write_raw_data(&compressed_file)?;
                    fs::remove_file(compressed_file)?;
                    println!(
                        "{:?} compressing and serializing complete!",
                        &self.original_file_list[i]
                    );
                }
                false => {
                    self.write_raw_data(&original_file)?;
                    println!("{:?} serializing complete!", &self.original_file_list[i]);
                }
            }
        }
        if PathBuf::from(TEMP_COMPRESSED_FILE_PATH).is_dir() {
            fs::remove_dir_all(TEMP_COMPRESSED_FILE_PATH)?;
        }
        self.result.flush()?;
        Ok(())
    }

    fn serialize_with_encrypt(&mut self, password: &str) -> io::Result<()> {
        let header = Header::with(true, false, self.original_file_list.len() as u64);
        self.result.write(&header.to_binary_vec())?;
        let (key, salt) = make_new_key_from_password(password);
        // Write salt.
        self.result.write(&salt)?;
        for i in 0..self.original_file_list.len() {
            // Write metadata.
            let mut metadata = MetaData::from(&self.original_file_list[i]);
            metadata.strip_prefix(&self.parent);
            self.result.write(&metadata.serialize())?;

            // Write binary data.
            let original_file = self.original_file_list[i].clone();
            match self.option.is_compressed() {
                true => {
                    let compressed_file =
                        compress::compress(original_file, TEMP_COMPRESSED_FILE_PATH)?;
                    self.result
                        .write(&compressed_file.metadata()?.len().to_le_bytes().to_vec())?;
                    self.write_encrypt_data(&compressed_file, &key)?;
                    fs::remove_file(compressed_file)?;
                    println!(
                        "{:?} compressing and serializing complete!",
                        &self.original_file_list[i]
                    );
                }
                false => {
                    self.write_encrypt_data(&original_file, &key)?;
                    println!("{:?} serializing complete!", &self.original_file_list[i]);
                }
            }
        }
        if PathBuf::from(TEMP_COMPRESSED_FILE_PATH).is_dir() {
            fs::remove_dir_all(TEMP_COMPRESSED_FILE_PATH)?;
        }
        self.result.flush()?;
        Ok(())
    }

    fn write_raw_data<T: AsRef<Path>>(&mut self, original_file: T) -> io::Result<()> {
        let mut buffer_reader = BufReader::new(File::open(original_file)?);
        loop {
            let length = {
                let buffer = buffer_reader.fill_buf()?;

                self.result.write(buffer)?;
                buffer.len()
            };
            if length == 0 {
                break;
            }
            buffer_reader.consume(length);
        }
        self.result.flush()?;
        Ok(())
    }

    fn write_encrypt_data<T: AsRef<Path>>(
        &mut self,
        original_file: T,
        key: &[u8],
    ) -> io::Result<()> {
        let mut buffer_reader = BufReader::with_capacity(BUFFER_LENGTH, File::open(original_file)?);
        let nonce = make_nonce();
        let mut encryptor = make_encryptor(key, &nonce);

        // Every time the encryption begins, create another random nonce.
        self.result.write(&nonce)?;

        let mut buffer = [0u8; BUFFER_LENGTH];
        loop {
            let length = buffer_reader.read(&mut buffer)?;
            if length == BUFFER_LENGTH {
                let encrypted_data = match encryptor.encrypt_next(buffer.as_slice()) {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Cannot encrypt data!",
                        ))
                    }
                };
                self.result.write(&encrypted_data)?;
            } else {
                let encrypted_data = match encryptor.encrypt_last(&buffer[..length]) {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Cannot encrypt data!",
                        ))
                    }
                };
                self.result.write(&encrypted_data)?;
                break;
            }
        }
        self.result.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::serialize::option::SerializeOption;

    use super::Serializer;
    use std::{fs, path::PathBuf};

    #[test]
    fn serialize_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(SerializeOption::default());
        serializer.serialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }

    #[test]
    fn serialize_with_encrypt_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_with_encrypt_test.bin");
        let option = SerializeOption::new().to_encrypt("test_password");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(option);
        serializer.serialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }

    #[test]
    fn serialize_with_compress_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_with_compress_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(SerializeOption::new().to_compress(true));
        serializer.serialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }

    #[test]
    fn serialize_with_encrypt_compress_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_with_encrypt_compress_test.bin");
        let option = SerializeOption::new()
            .to_encrypt("test_password")
            .to_compress(true);
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.set_option(option.clone());
        serializer.serialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }
}
