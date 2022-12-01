/// Option struct for serializing and deserializing
///
/// Pass this object to the serializer or deserializer.
///
/// #Examples
/// ```
/// use lusl::SerializeOption;
/// let default_option = SerializeOption::default();
/// assert_eq!(default_option.is_encrypted(), false);
/// assert_eq!(default_option.is_compressed(), false);
/// assert_eq!(default_option.password(), None);
/// let option = SerializeOption::new()
/// .to_encrypt("test_password")
/// .to_compress(true);
/// assert_eq!(option.is_encrypted(), true);
/// assert_eq!(option.is_compressed(), true);
/// assert_eq!(option.password(), Some(String::from("test_password")));
/// ```
#[derive(Clone)]
pub struct SerializeOption {
    encrypt: bool,
    compress: bool,
    password: Option<String>,
}

impl Default for SerializeOption {
    fn default() -> Self {
        Self {
            encrypt: false,
            compress: false,
            password: None,
        }
    }
}

impl SerializeOption {

    /// Make a new SerializeOption object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the encryption option.
    pub fn to_encrypt(mut self, password: &str) -> Self {
        self.encrypt = true;
        self.password = Some(String::from(password));
        self
    }

    /// Set the compression option.
    pub fn to_compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Returns true if the option is set to encrypt.
    pub fn is_encrypted(&self) -> bool {
        self.encrypt
    }

    /// Returns true if the option is set to compress.
    pub fn is_compressed(&self) -> bool {
        self.compress
    }

    /// Returns the password if the option is set to encrypt.
    pub fn password(&self) -> Option<String> {
        return self.password.clone();
    }
}
