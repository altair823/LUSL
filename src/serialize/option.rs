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
pub struct SerializeOption {
    encrypt: bool,
    compress: bool,
    password: Option<String>,
}

impl Default for SerializeOption {
    fn default() -> Self {
        Self { encrypt: false, compress: false, password: None }
    }
}

impl SerializeOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_encrypt(mut self, password: &str) -> Self {
        self.encrypt = true;
        self.password = Some(String::from(password));
        self
    }

    pub fn to_compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    pub fn is_encrypted(&self) -> bool {
        self.encrypt
    }

    pub fn is_compressed(&self) -> bool {
        self.compress
    }

    pub fn password(&self) -> Option<String> {
        return self.password.clone();
    }
}