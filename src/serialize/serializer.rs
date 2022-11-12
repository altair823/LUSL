use super::{get_file_list, meta::MetaData, BUFFERS_SIZE};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

pub struct Serializer {
    parent: PathBuf,
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
            parent: original_root.as_ref().parent().unwrap().to_path_buf(),
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
            let mut metadata = MetaData::from(file);
            metadata.strip_prifix(&self.parent);
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
            println!("{:?} serializing complete!", &file);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::Serializer;
    use std::{fs, path::PathBuf};

    #[test]
    fn serialize_test() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialize_test.bin");
        let mut manager = Serializer::new(original, result.clone()).unwrap();
        manager.serialize().unwrap();
        assert!(&result.is_file());
        if result.is_file() {
            fs::remove_file(result).unwrap();
        }
    }
}
