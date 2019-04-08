use crate::config::ConfigError;
use crate::index::TextIndexError;
use crate::worker::WorkerError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type ApplicationResult = Result<(), ApplicationError>;

#[derive(Debug)]
pub enum ApplicationError {
    ConfigError { message: String },
    TextIndexError { message: String },
    WorkerError { message: String },
}

impl ApplicationError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn config_error(error: ConfigError) -> ApplicationError {
        error!("Invalid configuration - {}", error);

        ApplicationError::ConfigError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn text_index_error(error: TextIndexError) -> ApplicationError {
        error!("Index initialization failed - {}", error);

        ApplicationError::TextIndexError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn worker_error(error: WorkerError) -> ApplicationError {
        error!("Worker initialization failed - {}", error);

        ApplicationError::WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn server_error<E>(error: E) -> ApplicationError
    where
        E: Error,
    {
        error!("Server error - {}", error);

        ApplicationError::WorkerError {
            message: format!("{}", error),
        }
    }
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ApplicationError::ConfigError { message } => write!(f, "{}", message),
            ApplicationError::TextIndexError { message } => write!(f, "{}", message),
            ApplicationError::WorkerError { message } => write!(f, "{}", message),
        }
    }
}
