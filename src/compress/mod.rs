use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use flate2::{
    bufread::{ZlibDecoder, ZlibEncoder},
    Compression,
};

pub const TEMP_COMPRESSED_FILE_PATH: &str = "./.LUSL_temp";

pub fn compress<T: AsRef<Path>, O: AsRef<Path>>(
    original_file_path: T,
    destination_path: O,
) -> io::Result<PathBuf> {
    let mut compressor = ZlibEncoder::new(
        BufReader::new(File::open(&original_file_path)?),
        Compression::new(9),
    );
    let dir = destination_path.as_ref().to_path_buf();
    fs::create_dir_all(&dir)?;
    let mut t = original_file_path.as_ref().to_path_buf();
    t.set_extension("zip");
    let compressed_file_path = dir.join(t.file_name().unwrap());
    let mut result = BufWriter::new(File::create(&compressed_file_path)?);
    let mut buf = Vec::new();
    compressor.read_to_end(&mut buf)?;
    result.write_all(&buf)?;
    result.flush()?;
    Ok(compressed_file_path)
}

pub fn decompress<T: AsRef<Path>, O: AsRef<Path>>(
    original_file_path: T,
    destination_path: O,
) -> io::Result<PathBuf> {
    let mut decompressor = ZlibDecoder::new(BufReader::new(File::open(&original_file_path)?));
    let dir = destination_path.as_ref().to_path_buf();
    fs::create_dir_all(&dir)?;
    let mut t = original_file_path.as_ref().to_path_buf();
    let e = t.clone();
    let original_extension = e.extension();
    t.set_extension("bin");
    let decompressed_file_path = dir.join(t.file_name().unwrap());
    let mut result = BufWriter::new(File::create(&decompressed_file_path)?);
    let mut buf = Vec::new();
    decompressor.read_to_end(&mut buf)?;
    result.write_all(&buf)?;
    result.flush()?;
    fs::remove_file(&original_file_path)?;
    let mut p = decompressed_file_path.clone();
    p.set_extension(match original_extension {
        Some(e) => e,
        None => OsStr::new("s"),
    });
    fs::rename(&decompressed_file_path, &p)?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{compress, decompress};

    #[test]
    fn compress_test() {
        let p = PathBuf::from("./decompress_test_temp/board-g43968feec_1920.zip");
        compress(
            "tests/original_images/dir1/board-g43968feec_1920.jpg",
            p.parent().unwrap(),
        )
        .unwrap();
        fs::create_dir_all(&p.parent().unwrap()).unwrap();
        let a = decompress(&p, &p.parent().unwrap().join("test")).unwrap();
        let original_size = PathBuf::from("tests/original_images/dir1/board-g43968feec_1920.jpg")
            .metadata()
            .unwrap()
            .len();
        let decompressed_size = a.metadata().unwrap().len();
        assert_eq!(original_size, decompressed_size);
        fs::remove_dir_all(&p.parent().unwrap()).unwrap();
    }
}
