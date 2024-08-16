use jsonrpsee::types::ErrorObjectOwned;
use std::fmt::Display;

#[derive(Debug)]
#[repr(i32)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    JsonRpcRequestError = 1000,
    Encoding(&'static str),
    InvalidRequest(&'static str),
    Script(i8),
    Vm(String),
}

impl From<Error> for ErrorObjectOwned {
    fn from(error: Error) -> Self {
        let code = match error {
            Error::JsonRpcRequestError => 1000,
            Error::Encoding(_) => 1001,
            Error::InvalidRequest(_) => 1002,
            Error::Script(_) => 1003,
            Error::Vm(_) => 1004,
        };
        let msg = match error {
            Error::JsonRpcRequestError => "".to_owned(),
            Error::Encoding(msg) | Error::InvalidRequest(msg) => msg.to_owned(),
            Error::Vm(msg) => msg,
            Error::Script(code) => format!("Script returns {}", code),
        };

        ErrorObjectOwned::owned(code, msg, None::<()>)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::JsonRpcRequestError => write!(f, "JsonRpcRequestError"),
            Error::Encoding(msg) => write!(f, "Encoding error: {}", msg),
            Error::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Error::Script(code) => write!(f, "Script returns {}", code),
            Error::Vm(msg) => write!(f, "VM error: {}", msg),
        }
    }
}
