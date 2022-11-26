use std::io;

use crate::binary::is_flag_true;

use super::binary_to_u64;

pub const FILE_LABEL: &str = "LUSL Serialized File";
const ENCRYPTED_FLAG: u8 = 0x80;
const COMPRESSED_FLAG: u8 = 0x40;

pub struct Header<'a> {
    label: &'a str,
    is_encrypted: bool,
    is_compressed: bool,
    file_count: u64,
}

impl<'a> Header<'a> {
    pub fn new() -> Self {
        Header {
            label: FILE_LABEL,
            is_encrypted: false,
            is_compressed: false,
            file_count: 0,
        }
    }

    pub fn with(is_encrypted: bool, is_compressed: bool, file_count: u64) -> Self {
        Header {
            label: FILE_LABEL,
            is_encrypted,
            is_compressed,
            file_count,
        }
    }
    
    pub fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    pub fn is_compressed(&self) -> bool {
        self.is_compressed
    }

    pub fn file_count(&self) -> u64 {
        self.file_count
    }

    pub fn to_binary_vec(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        binary.append(&mut self.label_to_binary());
        binary.append(&mut self.flag_to_binary());
        binary.append(&mut self.file_count_to_binary());
        binary
    }

    fn label_to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        for i in self.label.as_bytes() {
            binary.push(*i);
        }
        binary
    }

    fn flag_to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::with_capacity(1);
        let mut flag: u8 = 0x0;
        if let true = self.is_encrypted {
            flag += ENCRYPTED_FLAG;
        }
        if let true = self.is_compressed {
            flag += COMPRESSED_FLAG;
        }
        binary.push(flag);
        binary
    }

    fn file_count_to_binary(&self) -> Vec<u8> {
        let mut count_binary: Vec<u8> = Vec::new();
        let mut index = 0;
        for byte in self.file_count.to_be_bytes() {
            if byte == 0 {
                index += 1;
            } else {
                break;
            }
        }
        let file_count_bytes = self.file_count.to_le_bytes().len() - index;
        count_binary.push(file_count_bytes as u8);
        for i in &self.file_count.to_le_bytes()[..file_count_bytes as usize] {
            count_binary.push(*i);
        }
        count_binary
    }

    pub fn deserialize_label(&mut self, binary: &[u8]) -> io::Result<()> {
        let mut label = String::new();
        for i in binary {
            label.push(*i as char);
        }
        if let false = label == self.label {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Wrong file label!",
            ))
        } else {
            Ok(())
        }
    }

    pub fn deserialize_flag(&mut self, binary: &[u8]) {
        self.is_encrypted = is_flag_true(binary[0], ENCRYPTED_FLAG);
        self.is_compressed = is_flag_true(binary[0], COMPRESSED_FLAG);
    }

    pub fn deserialize_file_count(&mut self, binary: &[u8]) {
        self.file_count = binary_to_u64(binary);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_test() {
        let header = Header::with(true, false, 83);
        let header_binary = header.to_binary_vec();
        let mut new_header = Header::new();
        new_header
            .deserialize_label(&header_binary[..FILE_LABEL.as_bytes().len()])
            .unwrap();
        new_header.deserialize_flag(
            &header_binary[FILE_LABEL.as_bytes().len()..FILE_LABEL.as_bytes().len() + 1],
        );
        let file_count_byte_size = header_binary[FILE_LABEL.as_bytes().len() + 1];
        new_header.deserialize_file_count(
            &header_binary[FILE_LABEL.as_bytes().len() + 2
                ..FILE_LABEL.as_bytes().len() + 2 + file_count_byte_size as usize],
        );
        assert_eq!(new_header.is_encrypted, true);
        assert_eq!(new_header.is_compressed, false);
        assert_eq!(new_header.file_count, 83);
    }
}
