pub static ORIGINAL_DIR: &str = "tests/original_images";
pub static RESULT_DIR_PRIFIX: &str = "tests/result_images";

use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DirEnv {
    pub original: PathBuf,
    pub result: PathBuf,
}

fn get_original_dir() -> PathBuf {
    PathBuf::from(ORIGINAL_DIR)
}

fn get_result_dir() -> PathBuf {
    let mut i = 0;
    let mut result_dir = PathBuf::from(format!("{}{}", RESULT_DIR_PRIFIX, i));
    while result_dir.is_dir() {
        i += 1;
        result_dir = PathBuf::from(format!("{}{}", RESULT_DIR_PRIFIX, i));
    }
    result_dir
}

pub fn make_dir_env() -> DirEnv {
    let original_dir = get_original_dir();
    let result_dir = get_result_dir();
    create_dir_all(&result_dir).unwrap();
    DirEnv {
        original: original_dir,
        result: result_dir,
    }
}

pub fn clean(dir_env: DirEnv) {
    let result_dir = dir_env.result;
    remove_dir_all(&result_dir).unwrap();
    println!("Delete {} complete!", result_dir.to_str().unwrap());
}
