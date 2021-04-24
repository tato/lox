#[derive(Debug)]
pub struct ErrorInfo {
    line: usize,
    location: String,
    message: String,
}
impl ErrorInfo {
    // pub fn new(line: usize, location: &str, message: &str) -> Self {
    //     Self {
    //         line,
    //         location: location.into(),
    //         message: message.into(),
    //     }
    // }
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

pub struct ErrorReporter {
    _had_error: bool,
    _had_runtime_error: bool,
}
impl ErrorReporter {
    pub fn new() -> Self {
        ErrorReporter {
            _had_error: false,
            _had_runtime_error: false,
        }
    }
    pub fn _report(line: i32, location: &str, message: &str) {
        println!("[line {}] Error{}: {}", line, location, message);
    }
}
