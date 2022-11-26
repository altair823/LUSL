/// Option struct for serializing and deserializing
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

    pub fn is_encrypt(&self) -> bool {
        self.encrypt
    }

    pub fn is_compressed(&self) -> bool {
        self.compress
    }

    pub fn password(&self) -> Result<String, String> {
        match &self.password {
            Some(p) => Ok(p.to_string()),
            None => Err(String::from("No password!")),
        }
    }
}