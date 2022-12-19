//! Version module.
//!
//! This module contains the version struct and functions to get the current version of the library.
//! The version is stored in the header of the serialized file.
//! The version is also used to check if the serialized file is compatible with the current library.
//! The version is stored in the following format:
//! - Version start flag: 1 byte
//! - Major version: 1 byte
//! - Minor version: 1 byte
//! - Patch version: 1 byte
//!

use core::fmt;
use std::io;

const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
const PATCH_VERSION: &str = env!("CARGO_PKG_VERSION_PATCH");

/// Get the current major version of the library.
pub fn get_major_version() -> u8 {
    MAJOR_VERSION.parse().unwrap_or_default()
}

/// Get the current minor version of the library.
pub fn get_minor_version() -> u8 {
    MINOR_VERSION.parse().unwrap_or_default()
}

/// Get the current patch version of the library.
pub fn get_patch_version() -> u8 {
    PATCH_VERSION.parse().unwrap_or_default()
}

/// Version struct.
///
/// This struct is used to store the version of the serialized file or library.
/// The version is stored in the header of the serialized file.
/// The version is also used to check if the serialized file is compatible with the current library.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    /// Create a new version.
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Version {
            major,
            minor,
            patch,
        }
    }

    /// Get the major version.
    pub fn major(&self) -> u8 {
        self.major
    }

    /// Get the minor version.
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// Get the patch version.
    pub fn patch(&self) -> u8 {
        self.patch
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid version bytes.",
            ));
        }
        Ok(Version {
            major: bytes[0],
            minor: bytes[1],
            patch: bytes[2],
        })
    }
    pub fn to_bytes(&self) -> [u8; 3] {
        [self.major, self.minor, self.patch]
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::new(
            get_major_version(),
            get_minor_version(),
            get_patch_version(),
        )
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
