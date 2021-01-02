use crate::interpreter::CommandKind;
use serde::export::Formatter;
use std::error;
use std::io::Error;
use std::option::NoneError;

#[derive(Debug)]
pub enum Simple {
    Clothing,
    ItemNotFound,
    TooHeavy,
    Fatal,
    Guarded,
    NotClothing,
    PlayerNotFound,
    CannotAcceptGivenItem,
}

#[derive(Debug)]
pub enum EnnuiError {
    UnidentifiedError,
    FatalError(String),
    SimpleError(Simple),
    MessageError(String),
    ComplexError(Simple, String),
    ReturnToPlayer { src: CommandKind, msg: String },
    IoError(std::io::Error),
    NoneFound(std::option::NoneError),
}

impl std::fmt::Display for EnnuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EnnuiError::UnidentifiedError => write!(f, "Source contains no data"),
            EnnuiError::IoError(ref err) => err.fmt(f),
            EnnuiError::NoneFound(ref _err) => write!(f, "None error encountered"),
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for EnnuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnnuiError::UnidentifiedError => None,
            EnnuiError::IoError(_) => None,
            EnnuiError::NoneFound(_) => None,
            EnnuiError::SimpleError(e) => None,
            EnnuiError::MessageError(_) => None,
            EnnuiError::ComplexError(_, _) => None,
            EnnuiError::ReturnToPlayer { .. } => None,
            _ => None,
        }
    }
}

impl From<std::io::Error> for EnnuiError {
    fn from(err: Error) -> Self {
        EnnuiError::IoError(err)
    }
}

impl From<std::option::NoneError> for EnnuiError {
    fn from(err: std::option::NoneError) -> Self {
        EnnuiError::NoneFound(err)
    }
}

impl From<EnnuiError> for std::option::NoneError {
    fn from(_: EnnuiError) -> Self {
        use EnnuiError::*;
        std::option::NoneError
    }
}
