use crate::index::TextIndexError;
use postgres::Error as PostgresError;
use serde_yaml::Error as YamlError;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

pub type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Debug)]
pub struct WorkerError {
    message: String,
}

impl WorkerError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn io_error(error: IoError) -> WorkerError {
        warn!("IO error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn yaml_error(error: YamlError) -> WorkerError {
        warn!("YAML error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn connection_error(error: PostgresError) -> WorkerError {
        warn!("Connection error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn query_error(error: PostgresError) -> WorkerError {
        warn!("Query error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn stream_error(error: PostgresError) -> WorkerError {
        warn!("Stream error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn value_error(error: PostgresError) -> WorkerError {
        warn!("Value error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn text_index_error(error: TextIndexError) -> WorkerError {
        warn!("Text index error - {}", error);

        WorkerError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn field_not_found(name: &str) -> WorkerError {
        warn!("Field `{}` not found", name);

        WorkerError {
            message: format!("Field `{}` not found", name),
        }
    }
}

impl Error for WorkerError {}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
