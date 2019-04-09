#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod config;
mod error;
mod handler;
mod index;
mod options;
mod server;
mod worker;

use crate::error::ApplicationError;
use crate::error::ApplicationResult;
use crate::index::TextIndexRef;
use crate::options::Options;
use structopt::StructOpt;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> ApplicationResult {
    env_logger::init();

    let options = Options::from_args();
    let config = config::load(options.config_path()).map_err(ApplicationError::config_error)?;
    let index = TextIndexRef::new(&config, options.new_index())
        .map_err(ApplicationError::text_index_error)?;

    worker::start(&config, &index).map_err(ApplicationError::worker_error)?;
    server::start(&options, &config, &index)?;

    Ok(())
}
