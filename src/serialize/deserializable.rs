pub trait Deserialize {
    fn deserialize(binary: &Vec<u8>) -> Self;
}
