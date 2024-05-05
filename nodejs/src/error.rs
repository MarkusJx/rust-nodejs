pub struct NodeError {
    message: String,
    code: i32,
}

impl NodeError {
    pub fn new(message: String, code: i32) -> Self {
        Self { message, code }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> i32 {
        self.code
    }
}

impl std::fmt::Debug for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeError")
            .field("message", &self.message)
            .field("code", &self.code)
            .finish()
    }
}

impl std::fmt::Display for NodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message, self.code)
    }
}

impl std::error::Error for NodeError {}

pub type Result<T> = std::result::Result<T, NodeError>;
