static ORIGINAL_DIR: &str = "test/original_images";
static RESULT_DIR_PRIFIX: &str = "test/result_images";

mod setup {
    use std::path::PathBuf;

    use super::{ORIGINAL_DIR, RESULT_DIR_PRIFIX};

    pub struct DirEnv {
        original: PathBuf,
        result: PathBuf,
    }

    pub fn get_original_dir() -> PathBuf {
        PathBuf::from(ORIGINAL_DIR)
    }

    pub fn get_result_dir() -> PathBuf {
        let mut i = 0;
        let mut result_dir = PathBuf::from(format!("{}{}", RESULT_DIR_PRIFIX, i));
        while result_dir.is_dir() {
            i += 1;
            result_dir = PathBuf::from(format!("{}{}", RESULT_DIR_PRIFIX, i));
        }
        result_dir
    }
    
}