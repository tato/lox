#[derive(thiserror::Error, Debug)]
pub enum InterpretError {
    #[error(transparent)]
    Compile(#[from] CompileError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

#[derive(thiserror::Error, Debug)]
pub enum CompileError {}

#[derive(thiserror::Error, Debug)]
pub enum RuntimeError {
    #[error("Byte '{0}' does not map to any op code.")]
    InvalidOpcode(u8),
}
