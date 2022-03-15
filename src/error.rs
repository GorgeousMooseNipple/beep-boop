use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, BaseError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum BaseError {
    PaError(portaudio_rs::PaError),
    InputError(String),
    WindowCreation(String),
    SynthError(String),
    ConversionError(String),
    StreamError(String),
    GUIError(String),
    ThreadError(String),
}

impl std::fmt::Display for BaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseError::PaError(e) => e.fmt(f),
            BaseError::InputError(msg) => write!(f, "Input error: {}", msg),
            BaseError::WindowCreation(msg) => write!(f, "Window creation error: {}", msg),
            BaseError::SynthError(msg) => write!(f, "Synth error: {}", msg),
            BaseError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            BaseError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            BaseError::GUIError(msg) => write!(f, "GUI error: {}", msg),
            BaseError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
        }
    }
}

impl Error for BaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BaseError::PaError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<portaudio_rs::PaError> for BaseError {
    fn from(e: portaudio_rs::PaError) -> Self {
        BaseError::PaError(e)
    }
}

// impl From<alsa::Error> for BaseError {
//     fn from(e: alsa::Error) -> Self {
//         BaseError::AlsaError(e)
//     }
// }

// Unstable feature
// impl From<std::option::NoneError> for BaseError {
//     fn from(e: std::option::NoneError) -> Self {

//     }
// }
