use chacha20poly1305::{XChaCha20Poly1305, KeyInit, aead::{OsRng, stream::{self, Encryptor}, rand_core::RngCore}, AeadInPlace};

use crate::encryption::{make_key_from_password, make_nonce};

use super::{get_file_list, meta::MetaData, VERIFY_STRING, BUFFER_LENGTH};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
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
/// use lusl::Serializer;
/// use std::path::PathBuf;
/// use std::fs;
///
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized1.bin");
/// let mut serializer = Serializer::new(original, result.clone()).unwrap();
/// serializer.serialize().unwrap();
/// assert!(result.is_file());
/// ```

pub struct Serializer {
    parent: PathBuf,
    original_file_list: Vec<PathBuf>,
    result: BufWriter<File>,
    to_encrypt: bool,
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
                        "file already exists!",
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
            to_encrypt: false,
        })
    }

    /// Serialize root directory and copy it to result file.
    pub fn serialize(&mut self) -> io::Result<()> {
        self.result
            .write(&Serializer::make_verify_marker(
                self.original_file_list.len(),
            ))
            .unwrap();
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


    pub fn serialize_with_encrypt(&mut self, password: &str) -> io::Result<()> {
        self.result
            .write(&Serializer::make_verify_marker(
                self.original_file_list.len(),
            ))
            .unwrap();
        for file in &self.original_file_list {
            // Write metadata.
            let mut metadata = MetaData::from(file);
            metadata.set_file_encrypted(self.to_encrypt);
            metadata.strip_prefix(&self.parent);
            self.result.write(&metadata.serialize())?;

            // Write binary data.
            write_encrypt_data(file, &mut self.result, &password)?;
            println!("{:?} serializing complete!", &file);
        }
        self.result.flush()?;
        Ok(())
    }

    fn make_verify_marker(file_count: usize) -> Vec<u8> {
        let mut marker: Vec<u8> = Vec::new();
        for i in VERIFY_STRING.as_bytes() {
            marker.push(*i);
        }
        let mut index = 0;
        for byte in file_count.to_be_bytes() {
            if byte == 0 {
                index += 1;
            } else {
                break;
            }
        }
        let file_count_bytes = file_count.to_le_bytes().len() - index;
        marker.push(file_count_bytes as u8);
        for i in &file_count.to_le_bytes()[..file_count_bytes as usize] {
            marker.push(*i);
        }

        marker
    }
}

fn write_raw_data<T: AsRef<Path>>(original_file: T, destination: &mut BufWriter<File>) -> io::Result<()> {
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

fn write_encrypt_data<T: AsRef<Path>>(original_file: T, destination: &mut BufWriter<File>, password: &str) -> io::Result<()> {
    let original_file = File::open(original_file)?;
    let mut buffer_reader = BufReader::with_capacity(BUFFER_LENGTH, original_file);
    let nonce = make_nonce();
    let (key, salt) = make_key_from_password(password); // this part takes lots of time!
    let aead = XChaCha20Poly1305::new_from_slice(&key).unwrap();
    let mut encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    destination.write(&salt)?;
    destination.write(&nonce)?;

    loop {
        let length = {
            let buffer = buffer_reader.fill_buf()?;
            let encrypted_data = match encryptor.encrypt_next(buffer){
                Ok(c) => c,
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "Cannot encrypt data!"))
            };
            destination.write(&encrypted_data)?;
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

#[cfg(test)]
mod tests {

    use super::Serializer;
    use std::{fs, path::PathBuf};

    #[test]
    fn serialize_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_test.bin");
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
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
        let mut serializer = Serializer::new(original, result.clone()).unwrap();
        serializer.serialize_with_encrypt("test_password").unwrap();
    }
}
