# Lossless Uncompressed Serializer Library

[![Documentation](https://docs.rs/image/badge.svg)](https://docs.rs/lusl/)

A library for serializing and deserializing a directory. 

## Features

- Serialize a directory that contains multiple files. 
- Deserialize serialized file and restore to a directory. 
- Save and verify MD5 checksum of files for data integrity. 

## Supported File Format

- All

## Examples

### `Serializer` example
```rust
use lusl::Serializer;
use std::path::PathBuf;
use std::fs;
let original = PathBuf::from("tests");
let result = PathBuf::from("serialized.bin");
let mut serializer = Serializer::new(original, result.clone()).unwrap();
serializer.serialize().unwrap();
assert!(result.is_file());
```

### `Deserializer` example
```rust
use lusl::{Serializer, Deserializer};
use std::path::PathBuf;
let serialized_file = PathBuf::from("serialized.bin");
let restored = PathBuf::from("deserialized_dir");
let deserializer = Deserializer::new(serialized_file, restored.clone());
deserializer.deserialize().unwrap();
assert!(&result.is_file());
assert!(&restored.is_dir());
```
