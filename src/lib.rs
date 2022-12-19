//!
//! # Lossless Uncompressed Serializer Library
//!
//! `lusl` is a library that serializes a directory containing multiple files into a single file and also deserializes it, like a tarball.
//!
//! This library also provides a way to encrypt and compress the serialized file.
//!
//! The encryption is done using [XChaCha20-Poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305#XChaCha20-Poly1305_%E2%80%93_extended_nonce_variant)
//! and the compression is done using [zlib](https://en.wikipedia.org/wiki/Zlib).
//!
//! It also saves [MD5](md5) checksums when serializing files and verify it when deserializing file for data integrity.
//!
//! ## Usage
//!
//! ### Serializing and deserializing without encrypting or compressing.
//! ```rust
//! use lusl::{Serializer, Deserializer, SerializeOption};
//! use std::path::PathBuf;
//!
//! // Serialize a directory into a file.
//! let original = PathBuf::from("tests");
//! let result = PathBuf::from("serialized.bin");
//! let mut serializer = Serializer::new(&original, &result).unwrap();
//! serializer.serialize().unwrap();
//!
//! // Deserialize the file into a directory.
//! let restored = PathBuf::from("deserialized_dir");
//! let mut deserializer = Deserializer::new(&result, &restored).unwrap();
//! deserializer.deserialize().unwrap();
//!
//! assert!(&result.is_file());
//! assert!(&restored.is_dir());
//! ```
//!
//! ### Serializing and deserializing with encrypting and compressing.
//! ```rust
//! use lusl::{Serializer, Deserializer, SerializeOption};
//! use std::path::PathBuf;
//!
//! // Serialize a directory into a file.
//! let original = PathBuf::from("tests");
//! let result = PathBuf::from("serialized.bin");
//! let mut serializer = Serializer::new(&original, &result).unwrap();
//!
//! // Set the encryption key and compression option.
//! serializer.set_option(SerializeOption::new().to_encrypt("password").to_compress(true));
//! serializer.serialize().unwrap();
//!
//! // Deserialize the file into a directory.
//! let restored = PathBuf::from("deserialized_dir");
//! let mut deserializer = Deserializer::new(&result, &restored).unwrap();
//!
//! // Set the encryption key and compression option.
//! deserializer.set_option(SerializeOption::new().to_encrypt("password").to_compress(true));
//! deserializer.deserialize().unwrap();
//!
//! assert!(&result.is_file());
//! assert!(&restored.is_dir());
//! ```
//!

mod binary;
mod compress;
mod encrypt;
mod serialize;

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub use serialize::deserializer::Deserializer;
use serialize::header::{FILE_LABEL, VERSION_START_POINTER};
pub use serialize::option::SerializeOption;
pub use serialize::serializer::Serializer;
pub use serialize::version;

/// Reads the version of the serialized file.
/// # Errors
/// This function will return an error if the file is not a serialized file or the version is invalid.
/// # Examples
/// ```rust
/// use std::path::PathBuf;
/// use lusl::read_version;
/// use lusl::{Serializer, Deserializer};
/// use lusl::version::{Version, get_major_version, get_minor_version, get_patch_version};
///
/// // create a new serialized file.
/// let original = PathBuf::from("tests");
/// let result = PathBuf::from("serialized.bin");
/// let mut serializer = Serializer::new(&original, &result).unwrap();
/// serializer.serialize().unwrap();
///
/// // read the version of the serialized file.
/// let version = read_version(&result).unwrap();
/// assert_eq!(get_major_version(), version.major());
/// assert_eq!(get_minor_version(), version.minor());
/// assert_eq!(get_patch_version(), version.patch());
///
/// ```
pub fn read_version<T: AsRef<Path>>(filepath: T) -> io::Result<version::Version> {
    let mut file = File::open(filepath)?;
    let mut buffer: Vec<u8> = Vec::with_capacity(FILE_LABEL.len());
    buffer.resize(FILE_LABEL.len(), 0);
    file.read(&mut buffer)?;
    let mut version_buffer: Vec<u8> = Vec::with_capacity(4);
    version_buffer.resize(4, 0);
    file.read(&mut version_buffer)?;
    if version_buffer[0] != VERSION_START_POINTER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid version format",
        ));
    }
    let version = version::Version::from_bytes(&version_buffer[1..4])?;
    Ok(version)
}

#[cfg(test)]
mod tests {
    use crate::serialize::version::{get_major_version, get_minor_version, get_patch_version};

    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn test_read_version() {
        // create a new serialized file.
        let original = PathBuf::from("tests");
        let result = PathBuf::from("serialized.bin");
        let mut serializer = Serializer::new(&original, &result).unwrap();
        serializer.serialize().unwrap();

        // read version of it.
        let version = read_version(&result).unwrap();
        assert_eq!(
            version,
            version::Version::new(
                get_major_version(),
                get_minor_version(),
                get_patch_version()
            )
        );

        // delete the file.
        fs::remove_file(&result).unwrap();
    }
}
