pub mod serialize;

pub use serialize::serializer::Serializer;
pub use serialize::deserializer::Deserializer;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn a() {
        let original = PathBuf::from("tests");
        let result = PathBuf::from("/home/pi/file_manager/result.srl");
        let mut ser = Serializer::new(original, result.clone()).unwrap();
        ser.serialize().unwrap();

        let restored = PathBuf::from("restored");
        let deser = Deserializer::new(result, restored);
        deser.deserialize().unwrap();
    }   
}