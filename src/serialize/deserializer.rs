use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::serialize::meta::get_checksum;

use super::{binary_to_u64, meta::MetaData, VERIFY_STRING};

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
/// use lusl::{Serializer, Deserializer};
/// use std::path::PathBuf;
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized2.bin");
/// let mut serializer = Serializer::new(original, result.clone()).unwrap();
/// serializer.serialize().unwrap();
/// let serialized_file = PathBuf::from("serialized2.bin");
/// let restored = PathBuf::from("deserialized_dir");
/// let deserializer = Deserializer::new(serialized_file, restored.clone());
/// deserializer.deserialize().unwrap();
/// assert!(&result.is_file());
/// assert!(&restored.is_dir());
/// ```
pub struct Deserializer {
    serialized_file_path: PathBuf,
    restore_path: PathBuf,
}

impl Deserializer {
    /// Set serialized data file path and restored file path.
    pub fn new<T: AsRef<Path>>(serialized_file: T, restore_path: T) -> Self {
        Deserializer {
            serialized_file_path: serialized_file.as_ref().to_path_buf(),
            restore_path: restore_path.as_ref().to_path_buf(),
        }
    }

    fn fill_buf(buffer: &mut VecDeque<u8>, reader: &mut BufReader<File>) -> io::Result<usize> {
        buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
        reader.consume(buffer.len());
        Ok(buffer.len())
    }

    fn fill_buf_with_len(
        buffer: &mut VecDeque<u8>,
        reader: &mut BufReader<File>,
        length: usize,
    ) -> io::Result<Vec<u8>> {
        let buffer = buffer;
        while buffer.len() < length {
            Deserializer::fill_buf(buffer, reader)?;
        }
        Ok(buffer.drain(..length).collect())
    }

    /// Deserialize data file to directory.
    ///
    /// Checking [MD5](md5) checksum of files and if it is different, occur error.
    ///
    /// # Errors
    /// MD5 checksum of deserialized file is different from original checksum.
    pub fn deserialize(&self) -> io::Result<()> {
        let file = File::open(&self.serialized_file_path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = VecDeque::with_capacity(reader.capacity());

        // Verify marker.
        let marker =
            Deserializer::fill_buf_with_len(&mut buffer, &mut reader, VERIFY_STRING.len())?;
        match String::from_utf8(marker) {
            Ok(m) => match m == String::from(VERIFY_STRING) {
                true => (),
                false => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Wrong Data File!",
                    ))
                }
            },
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Wrong Data File!",
                ))
            }
        };

        // Read the number of all files.
        let original_file_count_bytes =
            Deserializer::fill_buf_with_len(&mut buffer, &mut reader, 1)?[0];
        let original_file_count = binary_to_u64(&Deserializer::fill_buf_with_len(
            &mut buffer,
            &mut reader,
            original_file_count_bytes as usize,
        )?);
        let mut current_file_count: u64 = 0;
        loop {
            let mut metadata = MetaData::new();

            // Restore file path
            let path_size_bin = Deserializer::fill_buf_with_len(&mut buffer, &mut reader, 2)?;
            let path_size = path_size_bin[0] as usize * 0x100 + path_size_bin[1] as usize;
            metadata.deserialize_path(&Deserializer::fill_buf_with_len(
                &mut buffer,
                &mut reader,
                path_size,
            )?);

            // Restore file type
            let flag_and_byte_count =
                Deserializer::fill_buf_with_len(&mut buffer, &mut reader, 1)?[0];
            metadata.deserialize_type(flag_and_byte_count);

            // Restore file size
            let size_count = (flag_and_byte_count & 0xF) as usize;
            metadata.deserialize_size(&Deserializer::fill_buf_with_len(
                &mut buffer,
                &mut reader,
                size_count,
            )?);

            // Restore checksum
            metadata.deserialize_checksum(&Deserializer::fill_buf_with_len(
                &mut buffer,
                &mut reader,
                32,
            )?);

            // Write file
            let file_path = self.restore_path.join(&metadata.path);
            fs::create_dir_all(self.restore_path.join(&metadata.path).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&metadata.path))?;
            let mut file = BufWriter::new(
                OpenOptions::new()
                    .append(true)
                    .write(true)
                    .open(&file_path)?,
            );
            let mut counter = 0;
            let size = metadata.size as usize;
            loop {
                counter += Deserializer::fill_buf(&mut buffer, &mut reader)?;
                if counter > size {
                    if size > buffer.len() {
                        file.write(&Vec::from(buffer.clone())[..buffer.len() - (counter - size)])?;
                        buffer.drain(..buffer.len() - (counter - size));
                    } else {
                        file.write(&Vec::from(buffer.clone())[..size])?;
                        buffer.drain(..size);
                    }
                    file.flush()?;
                    break;
                }

                file.write(&Vec::from(buffer.clone()))?;
                buffer.clear();
                if counter == size {
                    file.flush()?;
                    break;
                }
            }

            // Verify checksum
            let file = File::open(&file_path)?;
            let new_checksum = get_checksum(file);
            let a = metadata.checksum.unwrap();
            if new_checksum == a {
                println!("{} deserialize complete!", file_path.to_str().unwrap());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Wrong checksum!!!! {}, current: {}, metadata: {}",
                        file_path.to_str().unwrap(),
                        new_checksum,
                        a
                    ),
                ));
            }

            // Count file.
            current_file_count += 1;

            if buffer.len() == 0 {
                if Deserializer::fill_buf(&mut buffer, &mut reader)? == 0 {
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
        serializer.serialize().unwrap();

        let serialized_file = PathBuf::from("deserialize_test.bin");
        let restored = PathBuf::from("deserialize_test_dir");
        let deserializer = Deserializer::new(serialized_file, restored.clone());
        deserializer.deserialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
        if restored.is_dir() {
            fs::remove_dir_all(restored).unwrap();
        }
    }
}
