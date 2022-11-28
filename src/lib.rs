//!
//! # Lossless Uncompressed Serializer Library
//!
//! `lusl` is a library that serializes a directory containing multiple files into a single file and also deserializes it, like a tarball.
//!
//! It also save [MD5](md5) checksum when serializing files and verify it when deserializing file for data integrity.

mod binary;
mod encryption;
mod serialize;
pub use serialize::deserializer::Deserializer;
pub use serialize::option::SerializeOption;
pub use serialize::serializer::Serializer;
