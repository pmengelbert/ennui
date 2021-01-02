use std::error;
use serde::export::Formatter;
use std::io::Error;

#[derive(Debug)]
pub enum EnnuiError {
    Test1,
    Test2 { source: std::io::Error },
    Test3(std::io::Error),
    Test4(std::option::NoneError)
}

impl std::fmt::Display for EnnuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            EnnuiError::Test1 => {
                write!(f, "Source contains no data")
            }
            EnnuiError::Test2{ .. } => {
                write!(f, "Read error")
            }
            EnnuiError::Test3(ref err) => {
                err.fmt(f)
            }
            EnnuiError::Test4(ref _err) => {
                write!(f, "None error encountered")
            }
        }
    }
}

impl error::Error for EnnuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            EnnuiError::Test1 => None,
            EnnuiError::Test2 { ref source } => Some(source),
            EnnuiError::Test3(_) => None,
            EnnuiError::Test4(_) => None,
        }
    }
}

impl From<std::io::Error> for EnnuiError {
    fn from(err: Error) -> Self {
        EnnuiError::Test3(err)
    }
}

impl From<std::option::NoneError> for EnnuiError {
    fn from(err: std::option::NoneError) -> Self {
        EnnuiError::Test4(err)
    }
}
