use serde::Serialize;
use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    Audio(String),
    Io(String),
    Window(String),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Audio(_) => "The audio preview could not be started. Try again.".to_string(),
            Self::Io(_) => {
                "Local app data could not be accessed. Check folder permissions and try again."
                    .to_string()
            }
            Self::Window(_) => "The caption window could not be opened. Try again.".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audio(message) => write!(formatter, "audio error: {message}"),
            Self::Io(message) => write!(formatter, "io error: {message}"),
            Self::Window(message) => write!(formatter, "window error: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::Window(error.to_string())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        eprintln!("{error}");

        Self {
            message: error.user_message(),
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
