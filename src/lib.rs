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
pub use serialize::deserializer::Deserializer;
pub use serialize::option::SerializeOption;
pub use serialize::serializer::Serializer;
