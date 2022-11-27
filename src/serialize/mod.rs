use std::{
    io,
    path::{Path, PathBuf},
};

pub mod serializer;
pub mod deserializer;
pub mod option;
mod header;
pub mod meta;

const BUFFER_LENGTH: usize = 8192;

/// Find all files in the root directory in a recursive way.
/// The hidden files started with `.` will be not included in result.
fn get_file_list<O: AsRef<Path>>(root: O) -> io::Result<Vec<PathBuf>> {
    let mut image_list: Vec<PathBuf> = Vec::new();
    let mut file_list: Vec<PathBuf> = root
        .as_ref()
        .read_dir()?
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut i = 0;
    loop {
        if i >= file_list.len() {
            break;
        }
        if file_list[i].is_dir() {
            for component in file_list[i].read_dir()? {
                file_list.push(component.unwrap().path());
            }
        } else if file_list[i]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .collect::<Vec<_>>()[0]
            != '.'
        {
            image_list.push(file_list[i].to_path_buf());
        }
        i += 1;
    }

    Ok(image_list)
}

fn binary_to_u64(binary: &[u8]) -> u64 {
    let mut num: u64 = 0;
    let mut coef = 1;
    for i in binary {
        num += *i as u64 * coef;
        coef *= 0x100;
    }
    num
}
