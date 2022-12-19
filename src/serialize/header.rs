use std::io;

use crate::{
    binary::{binary_to_u64, is_flag_true},
    version::Version,
};

use super::version::{get_major_version, get_minor_version, get_patch_version};

pub const FILE_LABEL: &str = "LUSL Serialized File";
pub const VERSION_START_POINTER: u8 = 0x1;
const ENCRYPTED_FLAG: u8 = 0x80;
const COMPRESSED_FLAG: u8 = 0x40;

#[derive(Clone, Debug, PartialEq, Eq)]

pub struct Header {
    version: Version,
    is_encrypted: bool,
    is_compressed: bool,
    file_count: u64,
}

impl Header {
    /// Creates a new header with default values.
    pub fn new() -> Self {
        Header {
            version: Version::new(
                get_major_version(),
                get_minor_version(),
                get_patch_version(),
            ),
            is_encrypted: false,
            is_compressed: false,
            file_count: 0,
        }
    }

    /// Creates a new header from the given data.
    pub fn with(is_encrypted: bool, is_compressed: bool, file_count: u64) -> Self {
        Header {
            version: Version::new(
                get_major_version(),
                get_minor_version(),
                get_patch_version(),
            ),
            is_encrypted,
            is_compressed,
            file_count,
        }
    }

    /// Returns the Version of the header.
    pub fn version(&self) -> Version {
        self.version.clone()
    }

    /// Returns true if the file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.is_encrypted
    }

    /// Returns true if the file is compressed.
    pub fn is_compressed(&self) -> bool {
        self.is_compressed
    }

    /// Returns the number of files in the archive.
    pub fn file_count(&self) -> u64 {
        self.file_count
    }

    /// Converts the header into a binary vector.
    pub fn to_binary_vec(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        binary.append(&mut self.label_to_binary());
        binary.append(&mut self.version_to_binary());
        binary.append(&mut self.flag_to_binary());
        binary.append(&mut self.file_count_to_binary());
        binary
    }

    /// Converts the label to a binary vector.
    fn label_to_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        for i in FILE_LABEL.as_bytes() {
            binary.push(*i);
        }
        binary
    }

    /// Converts the version to a binary vector.
    fn version_to_binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        binary.push(VERSION_START_POINTER);
        let version = self.version.to_bytes();
        for i in version.iter() {
            binary.push(*i);
        }
        binary
    }

    /// The flag is a byte that contains the following information:
    /// - Bit 0: Encrypted
    /// - Bit 1: Compressed
    /// - Bit 2-7: Reserved
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

    /// Convert file count to binary vector.
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

    /// Read label of file from binary vector.
    pub fn deserialize_label(&mut self, binary: &[u8]) -> io::Result<()> {
        let mut label = String::new();
        for i in binary {
            label.push(*i as char);
        }
        if let false = label == FILE_LABEL {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Wrong file label!",
            ))
        } else {
            Ok(())
        }
    }

    /// Deserialize the version and set the header's version.
    pub fn deserialize_version(&mut self, binary: &[u8]) -> io::Result<()> {
        if binary[0] == VERSION_START_POINTER {
            self.version = Version::from_bytes(&binary[1..])?;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "There is no version data in the file. The file may be invalid or too old for current library version.",
            ))
        }
    }

    /// Deserialize the flag byte and set the header's flags.
    pub fn deserialize_flag(&mut self, binary: &[u8]) {
        self.is_encrypted = is_flag_true(binary[0], ENCRYPTED_FLAG);
        self.is_compressed = is_flag_true(binary[0], COMPRESSED_FLAG);
    }

    /// Deserialize the file count and set the header's file count.
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
        let mut counter = FILE_LABEL.as_bytes().len();
        new_header
            .deserialize_label(&header_binary[..counter])
            .unwrap();
        new_header
            .deserialize_version(&header_binary[counter..counter + 4])
            .unwrap();
        counter += 4;
        new_header.deserialize_flag(&header_binary[counter..counter + 1]);
        counter += 1;
        let file_count_byte_size = header_binary[counter];
        counter += 1;
        new_header.deserialize_file_count(
            &header_binary[counter..counter + file_count_byte_size as usize],
        );
        assert_eq!(new_header.is_encrypted, true);
        assert_eq!(new_header.is_compressed, false);
        assert_eq!(new_header.file_count, 83);
    }

    #[test]
    fn version_compare_test() {
        let version1 = Version::new(1, 0, 0);
        let version2 = Version::new(1, 0, 0);
        assert_eq!(version1, version2);
        let version3 = Version::new(1, 0, 1);
        assert!(version1 < version3);
        let version4 = Version::new(1, 1, 0);
        assert!(version4 > version1);
        let version5 = Version::new(2, 0, 0);
        assert!(version5 > version1);
    }
}
