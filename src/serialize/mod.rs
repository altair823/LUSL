pub mod deserializer;
pub mod meta;
pub mod serializer;
const BUFFERS_SIZE: usize = 1024;

#[cfg(test)]
mod tests {

    use std::{path::PathBuf, fs};

    use super::*;

    #[test]
    fn t() {
        let original = PathBuf::from("/Users/altair823/Desktop/testcase");
        let result = PathBuf::from("/Users/altair823/Desktop/testcase.srl");
        if result.is_file() {
            fs::remove_file(&result).unwrap();
        }
        let mut serializer = serializer::Serializer::new(original, result).unwrap();
        serializer.serialize().unwrap();
    }

    #[test]
    fn a() {
        let serial = PathBuf::from("/Users/altair823/Desktop/testcase.srl");
        let restore = PathBuf::from("restored");
        let deserializer = deserializer::Deserializer::new(serial, restore);
        deserializer.deserialize().unwrap();
    }
}
