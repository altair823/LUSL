use std::fs::File;
use std::path::Path;
use std::{ffi::OsString, time::SystemTime};

use crate::serialize::deserializable::Deserialize;
use crate::serialize::serializable::Serialize;

use chrono::{DateTime, Utc, Local};
use md5;

#[derive(Debug)]
pub struct MetaData {
    name: OsString,
    extension: OsString,
    size: u64,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    created: Option<SystemTime>,
    modified: Option<SystemTime>,
    checksum: Option<String>,
}

impl MetaData {
    pub fn new() -> MetaData {
        MetaData {
            name: OsString::new(),
            extension: OsString::new(),
            size: 0,
            is_file: false,
            is_dir: false,
            is_symlink: false,
            created: None,
            modified: None,
            checksum: None,
        }
    }

    pub fn make_checksum<T: AsRef<[u8]>>(&mut self, data: T) {
        self.checksum = Some(format!("{:x}", md5::compute(data)));
    }

    fn name_ex_to_binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        let mut name = self.name.to_str().unwrap().to_string();
        while name.len() + self.extension.len() > u16::MAX.into() {
            name.pop();
        }
        name.push('.');
        name.push_str(self.extension.to_str().unwrap());
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
        for byte in self.size.to_be_bytes(){
            if byte == 0{
                index += 1;
            } else {
                break;
            }
        }
        if (self.size.to_le_bytes().len() - index) as u8 > 15{
            flag_and_size += 15;
        } else {
            flag_and_size += (self.size.to_le_bytes().len() - index) as u8;
        }
        binary.push(flag_and_size);
        for i in &self.size.to_le_bytes()[..self.size.to_le_bytes().len() - index]{
            binary.push(*i);
        }

        binary
    }

    fn datetime_to_binary(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();

        let create_datetime: DateTime<Local> = match self.created {
            Some(c) => DateTime::from(c),
            None => Local::now(),
        };
        let modified_datetime: DateTime<Local> = match self.modified{
            Some(m) => DateTime::from(m),
            None => Local::now(),
        };
        

        binary
    }
}

impl<T: AsRef<Path>> From<&T> for MetaData {
    fn from(file_path: &T) -> Self {
        match File::open(&file_path) {
            Ok(file) => {
                return MetaData {
                    name: match file_path.as_ref().file_stem() {
                        Some(s) => s.to_os_string(),
                        None => OsString::new(),
                    },
                    extension: match file_path.as_ref().extension() {
                        Some(s) => s.to_os_string(),
                        None => OsString::new(),
                    },
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
                    created: match file.metadata() {
                        Ok(m) => match m.created() {
                            Ok(c) => Some(c),
                            Err(_) => None,
                        },
                        Err(_) => todo!(),
                    },
                    modified: match file.metadata() {
                        Ok(m) => match m.modified() {
                            Ok(m) => Some(m),
                            Err(_) => None,
                        },
                        Err(_) => todo!(),
                    },
                    checksum: None,
                }
            }
            Err(_) => return MetaData::new(),
        };
    }
}

impl PartialEq for MetaData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.extension == other.extension
            && self.size == other.size
            && self.is_file == other.is_file
            && self.is_dir == other.is_dir
            && self.is_symlink == other.is_symlink
            && self.created == other.created
            && self.modified == other.modified
            && self.checksum == other.checksum
    }
}

impl Serialize for MetaData {
    fn serialize(&self) -> Vec<u8> {
        let mut binary: Vec<u8> = Vec::new();
        binary.append(&mut self.name_ex_to_binary());
        binary.append(&mut self.type_size_to_binary());
        binary
    }
}

impl Deserialize for MetaData {
    fn deserialize(binary: &Vec<u8>) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use std::collections::binary_heap;
    use std::ops::Deref;
    use std::{fs, path::PathBuf};

    use crate::serialize::serializable::Serialize;
    use crate::test_util::setup;
    use crate::test_util::setup::{ORIGINAL_FILE1, ORIGINAL_FILE2};

    use super::{super::test_util, MetaData};
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
            meta.make_checksum(data);
            original_metadata_vec.push(meta);
        }
        let mut result_metadata_vec = Vec::new();
        for f in result_file_vec {
            let mut meta = MetaData::from(&f);
            let data = fs::read(f).unwrap();
            meta.make_checksum(data);
            result_metadata_vec.push(meta);
        }

        assert_eq!(original_metadata_vec, result_metadata_vec);

        setup::clean(dir_env);
    }

    #[test]
    fn name_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
        let meta2 = MetaData::from(&PathBuf::from(ORIGINAL_FILE2));
        assert_eq!(meta1.serialize()[0], 0);
        assert_eq!(meta1.serialize()[1], 25);
        assert_eq!(meta2.serialize()[0], 0);
        assert_eq!(meta2.serialize()[1], 10);

        let expected_meta1_bi: [u8; 25] = [
            98, 111, 97, 114, 100, 45, 103, 52, 51, 57, 54, 56, 102, 101, 101, 99, 95, 49, 57, 50,
            48, 46, 106, 112, 103,
        ];
        let meta1_binary = meta1.serialize();
        let type_size_index = meta1_binary[0] as usize * 0x100 + meta1_binary[1] as usize;
        assert_eq!(meta1.serialize().deref()[2..type_size_index + 2], expected_meta1_bi);
        let expected_meta2_bi: [u8; 10] = [237, 143, 173, 235, 176, 156, 46, 106, 112, 103];
        let meta2_binary = meta2.serialize();
        let type_size_index = meta2_binary[0] as usize * 0x100 + meta2_binary[1] as usize;
        assert_eq!(meta2.serialize().deref()[2..type_size_index + 2], expected_meta2_bi);
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
        assert_eq!(binary.deref()[name_end_index + 3..name_end_index + type_size_index + 3], [1, 244, 13]);
    }

    #[test]
    fn datetime_serialize_test() {
        let meta1 = MetaData::from(&PathBuf::from(ORIGINAL_FILE1));
        let binary = meta1.serialize();
        meta1.datetime_to_binary();
    }
}
