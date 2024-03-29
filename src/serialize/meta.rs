use std::fs::File;
use std::path::{Path, PathBuf};

use crate::binary::{binary_to_u64, get_checksum, is_flag_true};

const FILE_FLAG: u8 = 0x80;
const DIR_FLAG: u8 = 0x40;
const SYMLINK_FLAG: u8 = 0x20;

#[derive(Debug)]
pub struct MetaData {
    path: PathBuf,
    size: u64,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    checksum: Option<Vec<u8>>,
}

impl MetaData {
    pub fn new() -> MetaData {
        MetaData {
            path: PathBuf::new(),
            size: 0,
            is_file: false,
            is_dir: false,
            is_symlink: false,
            checksum: None,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn checksum(&self) -> &Option<Vec<u8>> {
        &self.checksum
    }

    pub fn strip_prefix<T: AsRef<Path>>(&mut self, root: T) {
        self.path = self.path.strip_prefix(root).unwrap().to_path_buf()
    }

    fn serialize_path(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        let mut name = self.path.to_str().unwrap().to_string();
        while name.len() > u16::MAX.into() {
            name.pop();
        }
        let length: u16 = name.len().try_into().unwrap();
        let length = length.to_be_bytes();
        binary.push(length[0]);
        binary.push(length[1]);

        for i in name.bytes() {
            binary.push(i);
        }

        binary
    }

    fn serialize_type_size(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();

        let mut flag_and_size: u8 = 0x0;
        if let true = self.is_file {
            flag_and_size += FILE_FLAG;
        }
        if let true = self.is_dir {
            flag_and_size += DIR_FLAG;
        }
        if let true = self.is_symlink {
            flag_and_size += SYMLINK_FLAG;
        }

        let mut index = 0;
        for byte in self.size.to_be_bytes() {
            if byte == 0 {
                index += 1;
            } else {
                break;
            }
        }
        let size_bytes_count = (self.size.to_le_bytes().len() - index) as u8;
        flag_and_size += size_bytes_count;
        binary.push(flag_and_size);
        for i in &self.size.to_le_bytes()[..size_bytes_count as usize] {
            binary.push(*i);
        }

        binary
    }

    fn serialize_checksum(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        match &self.checksum {
            Some(c) => {
                for i in c {
                    binary.push(*i);
                }
            }
            None => {
                for _ in 0..16 {
                    binary.push(0);
                }
            }
        }
        binary
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        binary.append(&mut self.serialize_path());
        binary.append(&mut self.serialize_type_size());
        binary.append(&mut self.serialize_checksum());
        binary
    }

    pub fn deserialize_path(&mut self, name_binary: &[u8]) {
        self.path = match String::from_utf8(name_binary.to_vec()) {
            Ok(n) => PathBuf::from(n),
            Err(_) => PathBuf::from("untitled.bin"),
        };
    }
    pub fn deserialize_type(&mut self, type_flag: u8) {
        self.is_file = is_flag_true(type_flag, FILE_FLAG);
        self.is_dir = is_flag_true(type_flag, DIR_FLAG);
        self.is_symlink = is_flag_true(type_flag, SYMLINK_FLAG);
    }

    pub fn deserialize_size(&mut self, size_binary: &[u8]) {
        self.size = binary_to_u64(size_binary);
    }

    pub fn deserialize_checksum(&mut self, checksum_binary: &[u8]) {
        self.checksum = Some(checksum_binary.to_vec());
    }
}

impl<T: AsRef<Path>> From<&T> for MetaData {
    fn from(file_path: &T) -> Self {
        match File::open(&file_path) {
            Ok(file) => {
                return MetaData {
                    path: file_path.as_ref().to_path_buf(),
                    size: match file.metadata() {
                        Ok(m) => m.len(),
                        Err(_) => 0,
                    },
                    is_file: match file.metadata() {
                        Ok(m) => m.is_file(),
                        Err(_) => false,
                    },
                    is_dir: match file.metadata() {
                        Ok(m) => m.is_dir(),
                        Err(_) => false,
                    },
                    is_symlink: match file.metadata() {
                        Ok(m) => m.is_symlink(),
                        Err(_) => false,
                    },
                    checksum: { Some(get_checksum(file)) },
                }
            }
            Err(_) => return MetaData::new(),
        };
    }
}

impl PartialEq for MetaData {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.size == other.size
            && self.is_file == other.is_file
            && self.is_dir == other.is_dir
            && self.is_symlink == other.is_symlink
            && self.checksum == other.checksum
    }
}

#[cfg(test)]
mod tests {

    use std::{collections::VecDeque, path::PathBuf};
    use hex::decode;

    use crate::serialize::get_file_list;

    use super::MetaData;

    const ORIGINAL_FILE: &str = "tests/original_images/dir1/board-g43968feec_1920.jpg";

    #[test]
    fn metadata_compare_test() {
        let original = PathBuf::from("tests");

        let original_file_vec = get_file_list(&original).unwrap();
        let mut original_metadata_vec = Vec::new();
        for f in original_file_vec {
            let meta = MetaData::from(&f);
            original_metadata_vec.push(meta);
        }
        // path clearance
        let mut original_metadata_vec: Vec<MetaData> = original_metadata_vec
            .iter()
            .map(|m| MetaData {
                path: PathBuf::from(m.path.file_name().unwrap()),
                size: m.size,
                is_file: m.is_file,
                is_dir: m.is_dir,
                is_symlink: m.is_symlink,
                checksum: Some(m.checksum.clone().unwrap()),
            })
            .collect();
        let mut result_metadata_vec = Vec::from([
            MetaData {
                path: PathBuf::from("colorful-2174045.png"),
                size: 464447,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("4e42993bfd2756df48b646d68433db1e").unwrap()),
            },
            MetaData {
                path: PathBuf::from("capsules-g869437822_1920.jpg"),
                size: 371728,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("60e191a914756ff7ae259e33f40f20da").unwrap()),
            },
            MetaData {
                path: PathBuf::from("board-g43968feec_1920.jpg"),
                size: 914433,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("37ca14866812327e1776d8cbb250501c").unwrap()),
            },
            MetaData {
                path: PathBuf::from("laboratory-g8f9267f5f_1920.jpg"),
                size: 6737,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("0c37be929cdc29b5ac0914104cda75aa").unwrap()),
            },
            MetaData {
                path: PathBuf::from("폭발.jpg"),
                size: 562560,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("4753aff9b06a34832ad1de0a69d5dcd3").unwrap()),
            },
            MetaData {
                path: PathBuf::from("digitization-1755812_1920.jpg"),
                size: 468460,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("4b6cab47e9193a4aebe4c8c6b7c88c1b").unwrap()),
            },
            MetaData {
                path: PathBuf::from("syringe-ge5e95bfe6_1920.jpg"),
                size: 253304,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("a7385d8a719c3036a857e21225c5bd6b").unwrap()),
            },
            MetaData {
                path: PathBuf::from("books-g6617d4d97_1920.jpg"),
                size: 564004,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("65aee1442129f56a0a6157c6b55f80c9").unwrap()),
            },
            MetaData {
                path: PathBuf::from("test-pattern-152459.png"),
                size: 55262,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("a09d4eab0326ba5403369035531f9308").unwrap()),
            },
            MetaData {
                path: PathBuf::from("tv-g87676cdfb_1280.png"),
                size: 1280855,
                is_file: true,
                is_dir: false,
                is_symlink: false,
                checksum: Some(decode("91517821bc6851b0d9abec5d5adea961").unwrap()),
            },
        ]);
        original_metadata_vec.sort_by_key(|m| m.path.clone());
        result_metadata_vec.sort_by_key(|m| m.path.clone());
        assert_eq!(original_metadata_vec, result_metadata_vec);
    }

    #[test]
    fn name_serialize_test() {
        let meta = MetaData::from(&PathBuf::from(ORIGINAL_FILE));
        assert_eq!(meta.serialize()[0], 0);
        assert_eq!(meta.serialize()[1], 52);

        let expected_meta1_bi: [u8; 52] = [
            116, 101, 115, 116, 115, 47, 111, 114, 105, 103, 105, 110, 97, 108, 95, 105, 109, 97,
            103, 101, 115, 47, 100, 105, 114, 49, 47, 98, 111, 97, 114, 100, 45, 103, 52, 51, 57,
            54, 56, 102, 101, 101, 99, 95, 49, 57, 50, 48, 46, 106, 112, 103,
        ];
        let meta1_binary = meta.serialize();
        let type_size_index = meta1_binary[0] as usize * 0x100 + meta1_binary[1] as usize;
        assert_eq!(&meta.serialize()[2..type_size_index + 2], expected_meta1_bi);
    }

    #[test]
    fn flag_size_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE));
        let binary = meta1.serialize();
        let name_end_index = binary[0] as usize * 0x100 + binary[1] as usize;
        let type_size = binary[name_end_index + 2];

        assert_eq!(type_size & 0x80, 0x80);
        assert_eq!(type_size & 0x40, 0);
        assert_eq!(type_size & 0x20, 0);

        let type_size_index = (type_size & 0xF) as usize;
        assert_eq!(type_size_index, 3);
        // 131 means it is a file, and the size takes 3 bytes.
        // And size bytes are little endian.
        assert_eq!(
            &binary[name_end_index + 3..name_end_index + type_size_index + 3],
            [1, 244, 13]
        );
    }

    #[test]
    fn checksum_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE));

        let binary = meta1.serialize();
        let name_end_index = binary[0] as usize * 0x100 + binary[1] as usize;
        let type_size = binary[name_end_index + 2];
        let type_size_index = (type_size & 0xF) as usize;

        let expected_checksum: [u8; 16] = [
            55, 202, 20, 134, 104, 18, 50, 126, 23, 118, 216, 203, 178, 80, 80, 28
        ];

        assert_eq!(
            &binary
                [name_end_index + type_size_index + 3..name_end_index + type_size_index + 3 + 16],
            expected_checksum
        );
    }

    #[test]
    fn metadata_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE));
        let mut binary = VecDeque::from_iter(meta1.serialize());

        println!("{:?}", meta1);

        let mut meta2 = MetaData::new();

        // Restore file path
        let path_size = binary[0] as usize * 0x100 + binary[1] as usize;
        binary.drain(..2);
        meta2.deserialize_path(&binary.drain(..path_size).collect::<Vec<u8>>());

        // Restore file type
        let flag_and_byte_count = binary.pop_front().unwrap();
        meta2.deserialize_type(flag_and_byte_count);

        // Restore file size
        let size_count = (flag_and_byte_count & 0xF) as usize;
        meta2.deserialize_size(&binary.drain(..size_count).collect::<Vec<u8>>());

        // Restore checksum
        meta2.deserialize_checksum(&binary.drain(..16).collect::<Vec<u8>>());

        assert_eq!(meta1, meta2);
    }
}
