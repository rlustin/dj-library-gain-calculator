use quick_xml;
use std;
use std::error::Error;

#[derive(Debug)]
pub enum AppError {
    GenericError(String),
}


impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      match *self {
          AppError::GenericError(ref message) => f.write_str(message),
      }
  }
}

impl Error for AppError {
  fn description(&self) -> &str {
      match *self {
          AppError::GenericError(ref message) => message.as_str(),
      }
  }
}

impl From<quick_xml::DeError> for AppError {
    fn from(error: quick_xml::DeError) -> AppError {
        AppError::GenericError(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> AppError {
        AppError::GenericError(error.to_string())
    }
}
