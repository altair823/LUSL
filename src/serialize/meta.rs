use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

const HASH_CHUNK_SIZE: usize = 128;

#[derive(Debug)]
pub struct MetaData {
    path: PathBuf,
    size: u64,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    checksum: Option<String>,
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

    pub fn with_data(
        filepath: PathBuf,
        size: u64,
        is_file: bool,
        is_dir: bool,
        is_symlink: bool,
        checksum: String,
    ) -> Self {
        let filepath = PathBuf::from(filepath);
        MetaData {
            path: filepath,
            size: size,
            is_file: is_file,
            is_dir: is_dir,
            is_symlink: is_symlink,
            checksum: Some(checksum),
        }
    }

    pub fn strip_prifix<T: AsRef<Path>>(&mut self, root: T) {
        self.path = self.path.strip_prefix(root).unwrap().to_path_buf()
    }

    fn name_ex_to_binary(&self) -> Vec<u8> {
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

    fn type_size_to_binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();

        let mut flag_and_size: u8 = 0;
        if let true = self.is_file {
            flag_and_size += 0x80;
        }
        if let true = self.is_dir {
            flag_and_size += 0x40;
        }
        if let true = self.is_symlink {
            flag_and_size += 0x20;
        }

        let mut index = 0;
        for byte in self.size.to_be_bytes() {
            if byte == 0 {
                index += 1;
            } else {
                break;
            }
        }
        flag_and_size += (self.size.to_le_bytes().len() - index) as u8;
        binary.push(flag_and_size);
        for i in &self.size.to_le_bytes()[..self.size.to_le_bytes().len() - index] {
            binary.push(*i);
        }

        binary
    }

    fn checksum_to_binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        match &self.checksum {
            Some(c) => {
                for i in c.as_bytes() {
                    binary.push(*i);
                }
            }
            None => {
                for _ in 0..32 {
                    binary.push(0);
                }
            }
        }
        binary
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        binary.append(&mut self.name_ex_to_binary());
        binary.append(&mut self.type_size_to_binary());
        binary.append(&mut self.checksum_to_binary());
        binary
    }

    pub fn deserialize(binary: &Vec<u8>) -> Self {
        let name_size = binary[0] as usize * 0x100 + binary[1] as usize;
        let name = match std::str::from_utf8(&binary[2..name_size + 2]) {
            Ok(s) => String::from(s),
            Err(_) => String::from("Untitle.txt"),
        };
        let type_size = binary[name_size + 2];

        let is_file;
        let is_dir;
        let is_symlink;
        match type_size & 0x80 {
            0 => is_file = false,
            _ => is_file = true,
        }
        match type_size & 0x40 {
            0 => is_dir = false,
            _ => is_dir = true,
        }
        match type_size & 0x20 {
            0 => is_symlink = false,
            _ => is_symlink = true,
        }

        let type_size_index = (type_size & 0xF) as usize;
        let mut size: u64 = 0;
        let mut coef = 1;
        for i in &binary[name_size + 3..name_size + type_size_index + 3] {
            size += *i as u64 * coef;
            coef *= 0x100;
        }

        let checksum = match std::str::from_utf8(
            &binary[name_size + type_size_index + 3..name_size + type_size_index + 3 + 32],
        ) {
            Ok("00000000000000000000000000000000") => None,
            Ok(c) => Some(c.to_string()),
            Err(_) => None,
        };

        MetaData {
            path: PathBuf::from(name),
            size: size,
            is_file: is_file,
            is_dir: is_dir,
            is_symlink: is_symlink,
            checksum: checksum,
        }
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
                    checksum: {
                        let mut hasher = Md5::new();
                        let mut buf_reader = BufReader::with_capacity(HASH_CHUNK_SIZE, file);
                        loop {
                            let length = {
                                let buf = buf_reader.fill_buf().unwrap();
                                hasher.update(buf);
                                buf.len()
                            };
                            if length == 0 {
                                break;
                            }
                            buf_reader.consume(length);
                        }
                        let a = hasher.finalize();
                        Some(format!("{:x}", a))
                    },
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

    use std::ops::Deref;
    use std::{fs, path::PathBuf};

    use crate::test_util::setup;
    use crate::test_util::setup::{ORIGINAL_FILE1, ORIGINAL_FILE2};

    use super::{super::super::test_util, MetaData};
    use fs_extra::dir;

    #[test]
    fn metadata_compare_test() {
        let dir_env = test_util::setup::make_dir_env();

        let mut copy_option = dir::CopyOptions::new();
        copy_option.overwrite = true;
        dir::copy(&dir_env.original, &dir_env.result, &copy_option).unwrap();

        let original_file_vec = test_util::get_file_list(&dir_env.original).unwrap();
        let result_file_vec = test_util::get_file_list(&dir_env.result).unwrap();
        let mut original_metadata_vec = Vec::new();
        for f in original_file_vec {
            let mut meta = MetaData::from(&f);
            let data = fs::read(f).unwrap();
            original_metadata_vec.push(meta);
        }
        let mut result_metadata_vec = Vec::new();
        for f in result_file_vec {
            let mut meta = MetaData::from(&f);
            let data = fs::read(f).unwrap();
            result_metadata_vec.push(meta);
        }

        // path clearance
        let original_metadata_vec: Vec<MetaData> = original_metadata_vec
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
        let result_metadata_vec: Vec<MetaData> = result_metadata_vec
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
        assert_eq!(original_metadata_vec, result_metadata_vec);

        setup::clean(dir_env);
    }

    #[test]
    fn name_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
        let meta2 = MetaData::from(&PathBuf::from(ORIGINAL_FILE2));
        assert_eq!(meta1.serialize()[0], 0);
        assert_eq!(meta1.serialize()[1], 52);
        assert_eq!(meta2.serialize()[0], 0);
        assert_eq!(meta2.serialize()[1], 37);

        let expected_meta1_bi: [u8; 52] = [
            116, 101, 115, 116, 115, 47, 111, 114, 105, 103, 105, 110, 97, 108, 95, 105, 109, 97,
            103, 101, 115, 47, 100, 105, 114, 49, 47, 98, 111, 97, 114, 100, 45, 103, 52, 51, 57,
            54, 56, 102, 101, 101, 99, 95, 49, 57, 50, 48, 46, 106, 112, 103,
        ];
        let meta1_binary = meta1.serialize();
        let type_size_index = meta1_binary[0] as usize * 0x100 + meta1_binary[1] as usize;
        assert_eq!(
            meta1.serialize().deref()[2..type_size_index + 2],
            expected_meta1_bi
        );
        let expected_meta2_bi: [u8; 37] = [
            116, 101, 115, 116, 115, 47, 111, 114, 105, 103, 105, 110, 97, 108, 95, 105, 109, 97,
            103, 101, 115, 47, 100, 105, 114, 49, 47, 237, 143, 173, 235, 176, 156, 46, 106, 112,
            103,
        ];
        let meta2_binary = meta2.serialize();
        let type_size_index = meta2_binary[0] as usize * 0x100 + meta2_binary[1] as usize;
        assert_eq!(
            meta2.serialize().deref()[2..type_size_index + 2],
            expected_meta2_bi
        );
    }

    #[test]
    fn flag_size_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
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
            binary.deref()[name_end_index + 3..name_end_index + type_size_index + 3],
            [1, 244, 13]
        );
    }

    #[test]
    fn checksum_serialize_test() {
        let mut meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
        let data = fs::read(ORIGINAL_FILE1).unwrap();

        let binary = meta1.serialize();
        let name_end_index = binary[0] as usize * 0x100 + binary[1] as usize;
        let type_size = binary[name_end_index + 2];
        let type_size_index = (type_size & 0xF) as usize;

        let expected_checksum: [u8; 32] = [
            51, 55, 99, 97, 49, 52, 56, 54, 54, 56, 49, 50, 51, 50, 55, 101, 49, 55, 55, 54, 100,
            56, 99, 98, 98, 50, 53, 48, 53, 48, 49, 99,
        ];

        assert_eq!(
            binary.deref()
                [name_end_index + type_size_index + 3..name_end_index + type_size_index + 3 + 32],
            expected_checksum
        );
    }

    #[test]
    fn metadata_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
        let binary = meta1.serialize();

        println!("{:?}", meta1);

        let meta2 = MetaData::deserialize(&binary);
        assert_eq!(meta1, meta2);
    }
}
