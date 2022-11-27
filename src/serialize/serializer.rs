use chacha20poly1305::{aead::stream, KeyInit, XChaCha20Poly1305};

use crate::encryption::{make_new_key_from_password, make_nonce};

use super::{
    get_file_list,
    header::Header,
    meta::MetaData,
    BUFFER_LENGTH, option::SerializeOption,
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
/// # Examples
/// ```
/// use lusl::{Serializer, SerializeOption};
/// use std::path::PathBuf;
/// use std::fs;
///
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized1.bin");
/// let mut serializer = Serializer::new(original, result.clone()).unwrap();
/// serializer.serialize(&SerializeOption::default()).unwrap();
/// assert!(result.is_file());
/// ```

pub struct Serializer {
    parent: PathBuf,
    original_file_list: Vec<PathBuf>,
    result: BufWriter<File>,
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
        })
    }

    /// Serialize root directory and copy it to result file.
    pub fn serialize(&mut self, option: &SerializeOption) -> io::Result<()> {
        match option.is_encrypted() {
            true => self.serialize_with_encrypt(&option.password().unwrap())?,
            false => self.serialize_raw()?,
        }
        Ok(())
    }

    fn serialize_raw(&mut self) -> io::Result<()> {
        let header = Header::with(false, false, self.original_file_list.len() as u64);
        self.result.write(&header.to_binary_vec())?;
        for file in &self.original_file_list {
            // Write metadata.
            let mut metadata = MetaData::from(file);
            metadata.strip_prefix(&self.parent);
            self.result.write(&metadata.serialize())?;

            // Write binary data.
            write_raw_data(file, &mut self.result)?;
            println!("{:?} serializing complete!", &file);
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
        for file in &self.original_file_list {
            // Write metadata.
            let mut metadata = MetaData::from(file);
            metadata.strip_prefix(&self.parent);
            self.result.write(&metadata.serialize())?;

            // Write binary data.
            write_encrypt_data(file, &mut self.result, &key)?;
            println!("{:?} serializing complete!", &file);
        }
        self.result.flush()?;
        Ok(())
    }
}

fn write_raw_data<T: AsRef<Path>>(
    original_file: T,
    destination: &mut BufWriter<File>,
) -> io::Result<()> {
    let original_file = File::open(original_file)?;
    let mut buffer_reader = BufReader::new(original_file);
    loop {
        let length = {
            let buffer = buffer_reader.fill_buf()?;

            destination.write(buffer)?;
            buffer.len()
        };
        if length == 0 {
            break;
        }
        buffer_reader.consume(length);
    }
    destination.flush()?;
    Ok(())
}

fn write_encrypt_data<T: AsRef<Path>>(
    original_file: T,
    destination: &mut BufWriter<File>,
    key: &Vec<u8>,
) -> io::Result<()> {
    let original_file = File::open(original_file)?;
    let mut buffer_reader = BufReader::with_capacity(BUFFER_LENGTH, original_file);
    let nonce = make_nonce();
    let aead = XChaCha20Poly1305::new_from_slice(&key).unwrap();
    let mut encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    // Every time the encryption begins, create another random nonce.
    destination.write(&nonce)?;

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
            }; // The bottle neck!
            destination.write(&encrypted_data)?;
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
            destination.write(&encrypted_data)?;
            break;
        }
    }
    destination.flush()?;
    Ok(())
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
        serializer.serialize(&SerializeOption::default()).unwrap();
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
        serializer.serialize(&option).unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }
}
