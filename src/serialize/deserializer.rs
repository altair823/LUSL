use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use crate::serialize::meta::get_checksum;

use super::{BUFFERS_SIZE, meta::MetaData};

pub struct Deserializer {
    serialized_file_path: PathBuf,
    restore_path: PathBuf,
}

impl Deserializer {
    pub fn new<T: AsRef<Path>>(serialized_file: T, restore_path: T) -> Self {
        Deserializer {
            serialized_file_path: serialized_file.as_ref().to_path_buf(),
            restore_path: restore_path.as_ref().to_path_buf(),
        }
    }

    pub fn deserialize(&self) -> io::Result<()> {
        let file = File::open(&self.serialized_file_path)?;
        let mut reader = BufReader::with_capacity(BUFFERS_SIZE, file);
        let mut buffer = VecDeque::with_capacity(BUFFERS_SIZE);
        loop {
            let mut metadata = MetaData::new();
            // Restore file path
            while buffer.len() < 2 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            let path_size = buffer[0] as usize * 0x100 + buffer[1] as usize;
            buffer.drain(..2);

            while buffer.len() < path_size {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            metadata.deserialize_path(&buffer.drain(..path_size).collect::<Vec<u8>>());

            // Restore file type
            while buffer.len() < 1 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            let flag_and_byte_count = match buffer.pop_front() {
                Some(f) => f,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Cannot deserialize file type and size!",
                    ))
                }
            };
            metadata.deserialize_type(flag_and_byte_count);

            // Restore file size
            let size_count = (flag_and_byte_count & 0xF) as usize;
            while buffer.len() < size_count {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            metadata.deserialize_size(flag_and_byte_count, &buffer.drain(..size_count).collect::<Vec<u8>>());

            // Restore checksum
            while buffer.len() < 32 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            metadata.deserialize_checksum(&buffer.drain(..32).collect::<Vec<u8>>());

            // Write file
            let file_path = self.restore_path.join(&metadata.path);
            fs::create_dir_all(self.restore_path.join(&metadata.path).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&metadata.path)).unwrap();
            let mut file = OpenOptions::new()
                .append(true)
                .write(true)
                .open(&file_path)?;
            let mut counter = buffer.len();
            file.write(&Vec::from(buffer.clone()))?;
            buffer.clear();
            let size = metadata.size as usize;
            loop {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
                counter += buffer.len();
                if counter > size {
                    file.write(&Vec::from(buffer.clone())[..BUFFERS_SIZE - (counter - size)])
                        .unwrap();
                    buffer.drain(..BUFFERS_SIZE - (counter - size));
                    break;
                }

                file.write(&Vec::from(buffer.clone()))?;
                buffer.clear();
                if counter == size {
                    break;
                }
            }

            // Verify checksum
            let file = File::open(&file_path)?;
            let new_checksum = get_checksum(file);
            if new_checksum == metadata.checksum.unwrap() {
                println!("{} deserialize complete!", file_path.to_str().unwrap());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Wrong checksum!!!! {}", file_path.to_str().unwrap()),
                ));
            }

            if buffer.len() == 0 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
                if buffer.len() == 0 {
                    break;
                } else {
                    continue;
                }
            }
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
