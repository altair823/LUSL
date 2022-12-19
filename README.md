# Lossless Uncompressed Serializer Library

[![Documentation](https://docs.rs/image/badge.svg)](https://docs.rs/lusl/)

`lusl` is a library that serializes a directory containing multiple files into a single file and also deserializes it, like a tarball.

## Features

- Serialize a directory that contains multiple files. 
- Deserialize serialized file and restore to a directory. 
- Save and verify MD5 checksum of files for data integrity. 
- Provides a way to encrypt and compress the serialized file.

The encryption is done using [XChaCha20-Poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305#XChaCha20-Poly1305_%E2%80%93_extended_nonce_variant) 
and the compression is done using [zlib](https://en.wikipedia.org/wiki/Zlib).

## File Structure

See [documents](structure_of_serialized_file.md). 

## Usage

### Serializing and deserializing without encrypting or compressing.
```rust
use lusl::{Serializer, Deserializer, SerializeOption};
use std::path::PathBuf;

// Serialize a directory into a file.
let original = PathBuf::from("tests");
let result = PathBuf::from("serialized.bin");
let mut serializer = Serializer::new(&original, &result).unwrap();
serializer.serialize().unwrap();

// Deserialize the file into a directory.
let restored = PathBuf::from("deserialized_dir");
let mut deserializer = Deserializer::new(&result, &restored).unwrap();
deserializer.deserialize().unwrap();

assert!(&result.is_file());
assert!(&restored.is_dir());
```
### Serializing and deserializing with encrypting and compressing.
```rust
use lusl::{Serializer, Deserializer, SerializeOption};
use std::path::PathBuf;

// Serialize a directory into a file.
let original = PathBuf::from("tests");
let result = PathBuf::from("serialized.bin");
let mut serializer = Serializer::new(&original, &result).unwrap();

// Set the encryption key and compression option.
serializer.set_option(SerializeOption::new().to_encrypt("password").to_compress(true));
serializer.serialize().unwrap();

// Deserialize the file into a directory.
let restored = PathBuf::from("deserialized_dir");
let mut deserializer = Deserializer::new(&result, &restored).unwrap();

// Set the encryption key and compression option.
deserializer.set_option(SerializeOption::new().to_encrypt("password").to_compress(true));
deserializer.deserialize().unwrap();

assert!(&result.is_file());
assert!(&restored.is_dir());
```

## Test

If you want to run test codes(like `cargo test`), must not run parallel test. 

It cause multiple error because all test codes were written without assuming parallel tests. 

To run test, run code below. 

```bash
cargo test -- --test-threads=1
```