use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

pub type QueryParserResult<T> = Result<T, QueryParserError>;

#[derive(Debug)]
pub struct QueryParserError {
    message: String,
}

impl QueryParserError {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new<S>(message: S) -> QueryParserError
    where
        S: AsRef<str>,
    {
        QueryParserError {
            message: message.as_ref().into(),
        }
    }
}

impl Error for QueryParserError {}

impl Display for QueryParserError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
