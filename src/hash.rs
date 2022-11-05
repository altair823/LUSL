use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use md5;

pub fn make_md5_from_file<O: AsRef<Path>>(dir: O) -> String {
    let data = fs::read(dir).unwrap();
    format!("{:x}", md5::compute(data))
}

pub fn make_md5_from_vec<O: AsRef<Path>>(dir: Vec<O>) -> HashMap<String, String>
where
    PathBuf: From<O>,
{
    let mut hashes = HashMap::new();
    for d in dir {
        let p = PathBuf::from(d);
        let data = fs::read(&p).unwrap();
        hashes.insert(
            String::from(OsString::from(p.file_name().unwrap()).to_str().unwrap()),
            format!("{:x}", md5::compute(data)),
        );
    }
    hashes
}
