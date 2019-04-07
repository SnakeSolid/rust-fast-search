mod error;
mod postgres;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;
pub use self::postgres::PostgresWorker;

use crate::config::ConfigRef;
use crate::index::TextIndexRef;
use std::thread::Builder;

pub fn start(config: &ConfigRef, index: &TextIndexRef) -> WorkerResult<()> {
    let worker = PostgresWorker::new(config, index)?;

    Builder::new()
        .name("Query index worker".into())
        .spawn(move || worker.run())
        .map_err(WorkerError::io_error)?;

    Ok(())
}
