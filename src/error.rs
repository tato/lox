#[derive(Debug)]
pub struct ErrorInfo {
    line: usize,
    location: String,
    message: String,
}
impl ErrorInfo {
    pub fn new(line: usize, location: &str, message: &str) -> Self {
        Self {
            line,
            location: location.into(),
            message: message.into(),
        }
    }
    pub fn line(line: usize, message: &str) -> Self {
        Self {
            line,
            location: "".into(),
            message: message.into(),
        }
    }
}
impl std::fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, self.location, self.message
        )
    }
}
impl std::error::Error for ErrorInfo {}
