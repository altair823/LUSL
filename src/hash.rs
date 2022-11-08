use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use md5::{Md5, Digest};

pub fn make_md5_from_file<O: AsRef<Path>>(dir: O) -> String {
    let data = fs::read(dir).unwrap();
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn make_md5_from_vec<O: AsRef<Path>>(dir: Vec<O>) -> HashMap<String, String>
where
    PathBuf: From<O>,
{
    let mut hashes = HashMap::new();
    for d in dir {
        let p = PathBuf::from(d);
        let data = fs::read(&p).unwrap();
        let mut hasher = md5::Md5::new();
        hasher.update(data);
        hashes.insert(
            String::from(OsString::from(p.file_name().unwrap()).to_str().unwrap()),
            format!("{:x}", hasher.finalize())
        );
    }
    hashes
}
