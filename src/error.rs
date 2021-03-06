use std::error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CmdErr {
    Clothing,
    ItemNotFound,
    TooHeavy,
    Guarded,
    NotClothing,
    PlayerNotFound,
    CannotAcceptGivenItem,
}

#[derive(Debug)]
pub enum EnnuiError {
    Unidentified,
    Fatal(String),
    Lesser(String),
    Simple(CmdErr),
    Msg(String),
    NoneFound(std::option::NoneError),
    Quit,
}

impl Display for EnnuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EnnuiError::Unidentified => write!(f, "Source contains no data"),
            EnnuiError::NoneFound(ref _err) => write!(f, "None error encountered"),
            e => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for EnnuiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EnnuiError::Unidentified => None,
            EnnuiError::NoneFound(_) => None,
            EnnuiError::Simple(_e) => None,
            EnnuiError::Msg(_) => None,
            // EnnuiError::Complex(_, _) => None,
            _ => None,
        }
    }
}

impl From<std::option::NoneError> for EnnuiError {
    fn from(err: std::option::NoneError) -> Self {
        EnnuiError::NoneFound(err)
    }
}

impl From<EnnuiError> for std::option::NoneError {
    fn from(_: EnnuiError) -> Self {
        std::option::NoneError
    }
}
