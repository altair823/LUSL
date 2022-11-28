use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use md5::{Digest, Md5};

use crate::serialize::meta::MetaData;

pub fn is_flag_true(data: u8, flag: u8) -> bool {
    match data & flag {
        0 => false,
        _ => true,
    }
}

pub fn get_checksum(file: File) -> String {
    let mut hasher = Md5::new();
    let mut buf_reader = BufReader::new(file);
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
    format!("{:x}", a)
}

pub fn verify_checksum<T: AsRef<Path>>(metadata: MetaData, file_path: T) -> io::Result<()> {
    let file = File::open(&file_path)?;
    let new_checksum = get_checksum(file);
    let old_checksum = metadata.checksum().as_ref().unwrap();
    if new_checksum == *old_checksum {
        println!(
            "{} deserialize complete!",
            file_path.as_ref().to_str().unwrap()
        );
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Wrong checksum!!!! {}, new checksum: {}, old checksum: {}",
                file_path.as_ref().to_str().unwrap(),
                new_checksum,
                old_checksum
            ),
        ));
    }
    Ok(())
}
