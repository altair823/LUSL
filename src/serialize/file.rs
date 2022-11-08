

#[cfg(test)]
mod tests {

    use super::super::super::test_util::setup::ORIGINAL_FILE1;
    use super::super::meta;
    use std::{fs::{self, File}, io::{BufReader, BufRead}, path::PathBuf};

    fn serialize_file_with_metadata() -> Vec<u8>{
        let path = PathBuf::from(ORIGINAL_FILE1);
        let metadata = meta::MetaData::from(&path);

    }

    #[test]
    fn file_read_test(){
        // how to read large file?
        let mut file = File::open(ORIGINAL_FILE1).unwrap();
        let buffer_size = 10;
        let mut buf = BufReader::with_capacity(buffer_size, file);
        let data = buf.fill_buf().unwrap();
        println!("{}, {:?}", data.len(), data);
    }
}