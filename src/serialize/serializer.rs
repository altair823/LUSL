use super::{meta::MetaData, BUFFERS_SIZE};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

/// Find all files in the root directory in a recursive way.
/// The hidden files started with `.` will be not inclused in result.
pub fn get_file_list<O: AsRef<Path>>(root: O) -> io::Result<Vec<PathBuf>> {
    let mut image_list: Vec<PathBuf> = Vec::new();
    let mut file_list: Vec<PathBuf> = root
        .as_ref()
        .read_dir()?
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut i = 0;
    loop {
        if i >= file_list.len() {
            break;
        }
        if file_list[i].is_dir() {
            for component in file_list[i].read_dir()? {
                file_list.push(component.unwrap().path());
            }
        } else if file_list[i]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .collect::<Vec<_>>()[0]
            != '.'
        {
            image_list.push(file_list[i].to_path_buf());
        }
        i += 1;
    }

    Ok(image_list)
}

pub struct Serializer {
    original_file_list: Vec<PathBuf>,
    result: File,
}

impl Serializer {
    pub fn new<T: AsRef<Path>>(original_root: T, result_path: T) -> io::Result<Self> {
        let result_path = PathBuf::from(result_path.as_ref());
        if result_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "The result file already exist!",
            ));
        }
        File::create(&result_path)?;
        Ok(Serializer {
            original_file_list: get_file_list(original_root)?,
            result: OpenOptions::new()
                .append(true)
                .write(true)
                .open(result_path)?,
        })
    }

    pub fn serialize(&mut self) -> io::Result<()> {
        for file in &self.original_file_list {
            let original_file = File::open(file)?;

            // Write metadata.
            let metadata = MetaData::from(file);
            self.result.write(&metadata.serialize())?;

            let mut buffer_reader = BufReader::with_capacity(BUFFERS_SIZE, original_file);
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::{super::super::test_util::setup::ORIGINAL_FILE1, Serializer};
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
    };

    fn serialize_file_with_metadata() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("test.bin");
        let mut manager = Serializer::new(original, result).unwrap();
        manager.serialize().unwrap();
    }

    #[test]
    fn file_read_test() {
        // how to read large file?
        let file = File::open(ORIGINAL_FILE1).unwrap();
        let buffer_size = 10;
        let mut buf = BufReader::with_capacity(buffer_size, file);
        let data = buf.fill_buf().unwrap();
        println!("{}, {:?}", data.len(), data);
    }
}
