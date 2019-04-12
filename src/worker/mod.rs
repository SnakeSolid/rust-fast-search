mod error;
mod postgres;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;
pub use self::postgres::PostgresWorker;

use crate::config::ConfigRef;
use crate::index::TextIndexRef;
use std::fs::remove_file;
use std::thread::Builder;

pub fn start(config: &ConfigRef, index: &TextIndexRef, new_index: bool) -> WorkerResult<()> {
    if new_index {
        let state_file = config.state_file();

        if state_file.exists() {
            remove_file(state_file).map_err(WorkerError::io_error)?;
        }
    }

    let worker = PostgresWorker::new(config, index)?;

    Builder::new()
        .name("Query index worker".into())
        .spawn(move || worker.run())
        .map_err(WorkerError::io_error)?;

    Ok(())
}
