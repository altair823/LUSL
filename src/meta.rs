use std::fs::File;
use std::path::Path;
use std::{ffi::OsString, time::SystemTime};

use super::hash::make_md5_from_file;

#[derive(Debug)]
pub struct MetaData {
    name: OsString,
    extension: OsString,
    size: u64,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    created: SystemTime,
    modified: SystemTime,
    checksum: String,
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
            created: SystemTime::now(),
            modified: SystemTime::now(),
            checksum: String::new(),
        }
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
                            Ok(c) => c,
                            Err(_) => SystemTime::now(),
                        },
                        Err(_) => todo!(),
                    },
                    modified: match file.metadata() {
                        Ok(m) => match m.modified() {
                            Ok(m) => m,
                            Err(_) => SystemTime::now(),
                        },
                        Err(_) => todo!(),
                    },
                    checksum: make_md5_from_file(&file_path),
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

#[cfg(test)]
mod tests {

    use crate::test_util::setup;

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
            original_metadata_vec.push(MetaData::from(&f));
        }
        let mut result_metadata_vec = Vec::new();
        for f in result_file_vec {
            result_metadata_vec.push(MetaData::from(&f));
        }

        assert_eq!(original_metadata_vec, result_metadata_vec);

        setup::clean(dir_env);
    }
}
