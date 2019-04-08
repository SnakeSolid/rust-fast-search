use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use tantivy::directory::error::OpenDirectoryError;
use tantivy::TantivyError;

pub type TextIndexResult<T> = Result<T, TextIndexError>;

#[derive(Debug)]
pub struct TextIndexError {
    message: String,
}

impl TextIndexError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn open_directory_error(error: OpenDirectoryError) -> TextIndexError {
        warn!("Open directory error - {}", error);

        TextIndexError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn tantivy_error(error: TantivyError) -> TextIndexError {
        warn!("Tantivy error - {}", error);

        TextIndexError {
            message: format!("{}", error),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn poison_error<E>(error: E) -> TextIndexError
    where
        E: Error,
    {
        warn!("Poison error - {}", error);

        TextIndexError {
            message: format!("{}", error),
        }
    }
}

impl Error for TextIndexError {}

impl Display for TextIndexError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
