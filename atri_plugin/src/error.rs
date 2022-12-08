use std::fmt::{Debug, Display, Formatter};

pub type AtriResult<T> = Result<T, AtriError>;

#[derive(Debug)]
pub enum AtriError {
    JoinError(String),
    ClientError(String),
    SerializationError(String),
    NotSupported,
}

impl Display for AtriError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AtriError {}
