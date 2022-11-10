use std::{
    collections::VecDeque,
    fs::{File, self, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use super::BUFFERS_SIZE;

struct Deserialize {
    serialized_file_path: PathBuf,
    restore_path: PathBuf,
}

impl Deserialize {
    pub fn new<T: AsRef<Path>>(serialized_file: T, restore_path: T) -> Self {
        Deserialize {
            serialized_file_path: serialized_file.as_ref().to_path_buf(),
            restore_path: restore_path.as_ref().to_path_buf(),
        }
    }

    pub fn deserialize(&self) -> io::Result<()> {
        let file = File::open(&self.serialized_file_path)?;
        let mut reader = BufReader::with_capacity(BUFFERS_SIZE, file);
        let mut buffer = VecDeque::with_capacity(BUFFERS_SIZE);
        let mut consume_length = 0;
        loop {
            // Restore file name
            while buffer.len() < 2 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            let name_size = buffer[0] as usize * 0x100 + buffer[1] as usize;
            buffer.pop_front();
            buffer.pop_front();

            while buffer.len() < name_size {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            let mut name_buffer = Vec::new();
            for i in 0..name_size {
                name_buffer.push(buffer.pop_front().unwrap());
            }
            let name = match String::from_utf8(name_buffer) {
                Ok(n) => n,
                Err(_) => String::from("untitle.bin"),
            };

            // Restore file type and file size
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
            let is_file;
            let is_dir;
            let is_symlink;
            match flag_and_byte_count & 0x80 {
                0 => is_file = false,
                _ => is_file = true,
            }
            match flag_and_byte_count & 0x40 {
                0 => is_dir = false,
                _ => is_dir = true,
            }
            match flag_and_byte_count & 0x20 {
                0 => is_symlink = false,
                _ => is_symlink = true,
            }
            let size_count = (flag_and_byte_count & 0xF) as usize;

            while buffer.len() < size_count {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            let mut size: u64 = 0;
            let mut coef = 1;
            for _ in 0..size_count {
                size += buffer.pop_front().unwrap() as u64 * coef;
                coef *= 0x100;
            }

            // Restore checksum
            let mut checksum = String::new();
            while buffer.len() < 32 {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
            }
            for _ in 0..32 {
                checksum.push(buffer.pop_front().unwrap() as char);
            }

            // Write file
            let file_path = self.restore_path.join(&name);
            fs::create_dir_all(self.restore_path.join(&name).parent().unwrap()).unwrap();
            File::create(self.restore_path.join(&name)).unwrap();
            let mut file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(file_path)?;
            let mut counter = buffer.len();
            let mut temp = Vec::from(buffer.clone());
            file.write(&temp)?;

            counter += buffer.len();
            buffer.clear();
            let size = size as usize;
            while counter < size {
                buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
                reader.consume(buffer.len());
                let temp = Vec::from(buffer.clone());
                file.write(&temp)?;
                counter += buffer.len();
                buffer.clear();
            }
            buffer.append(&mut VecDeque::from_iter(reader.fill_buf()?.to_vec()));
            reader.consume(buffer.len());
            let temp = Vec::from(buffer.clone());
            file.write(&temp[..counter - size]).unwrap();
            for i in 0..counter - size {
                buffer.pop_front();
            }

            if buffer.len() == 0 {
                break;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn deserialize_file_test() {
        let serialized_file = PathBuf::from("test.bin");
        let restored = PathBuf::from("restored");
        let mut manager = Deserialize::new(serialized_file, restored);
        manager.deserialize().unwrap();
    }
}
