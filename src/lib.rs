pub mod hash;
pub mod meta;
pub mod test_util;

#[cfg(test)]
mod tests {
    use fs_extra::dir;

    use md5;

    use super::test_util::setup;
    use crate::test_util::{get_original_md5, get_result_md5};

    #[test]
    fn md5_test() {
        let s = b"hello world!";
        assert_eq!(
            format!("{:x}", md5::compute(s)),
            "fc3ff98e8c6a0d3087d515c0473f8677"
        );

        let original_hash = get_original_md5();

        let dir_env = setup::make_dir_env();

        let mut copy_option = dir::CopyOptions::new();
        copy_option.overwrite = true;
        dir::copy(&dir_env.original, &dir_env.result, &copy_option).unwrap();

        let result_hash = get_result_md5(&dir_env.result);
        assert_eq!(original_hash, result_hash);

        setup::clean(dir_env);
    }
}
