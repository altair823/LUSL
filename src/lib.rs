mod test_util;
#[cfg(test)]
mod tests {

    use std::io::{Read, self};
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use  std::fs;

    use md5;

    /// Find all files in the root directory in a recursive way.
    /// The hidden files started with `.` will be not inclused in result.
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

    fn get_dir_md5<O: AsRef<Path>>(dir: Vec<O>) -> HashMap<PathBuf, String> 
    where PathBuf: From<O> {
        let mut hashes: HashMap<PathBuf, String> = HashMap::new();
        for d in dir {
            let p = PathBuf::from(d);
            let data = fs::read(&p).unwrap();
            hashes.insert(p, format!("{:x}", md5::compute(data)));
        }
        hashes
    }

    fn get_original_md5() -> HashMap<PathBuf, String> {
        get_dir_md5(get_file_list("tests/original_images").unwrap())
    }

    fn get_result_md5<O: AsRef<Path>>(dir: O) -> HashMap<PathBuf, String> {
        get_dir_md5(get_file_list(dir).unwrap())
    }

    #[test]
    fn md5_test() {
        let s = b"hello world!";
        assert_eq!(format!("{:x}", md5::compute(s)), "fc3ff98e8c6a0d3087d515c0473f8677");

        
        println!("{:?}", get_original_md5());
    }

}
