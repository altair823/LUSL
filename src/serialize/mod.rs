pub mod deserializer;
pub mod meta;
pub mod serializer;
const BUFFERS_SIZE: usize = 1024;


#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;

    #[test]
    fn t() {
        let original = PathBuf::from("C:/Users/rlaxo/Desktop/실록");
        let result = PathBuf::from("C:/Users/rlaxo/Desktop/실록.srl");
        let mut serializer = serializer::Serializer::new(original, result).unwrap();
        serializer.serialize().unwrap();
    }

    #[test]
    fn a() {
        let serial = PathBuf::from("실록.srl");
        let restore = PathBuf::from("restored");
        let deserializer = deserializer::Deserializer::new(serial, restore);
        deserializer.deserialize().unwrap();
    }
}