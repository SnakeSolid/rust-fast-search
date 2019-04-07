#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod config;
mod error;
mod index;
mod options;
mod worker;

use crate::error::ApplicationError;
use crate::error::ApplicationResult;
use crate::index::TextIndexRef;
use crate::options::Options;
use std::io;
use std::io::BufRead;
use std::io::Write;
use structopt::StructOpt;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> ApplicationResult {
    env_logger::init();

    let options = dbg!(Options::from_args());
    let config = config::load(options.config_path()).map_err(ApplicationError::config_error)?;
    let index = TextIndexRef::new(&config, options.new_index())
        .map_err(ApplicationError::text_index_error)?;

    worker::start(&config, &index).map_err(ApplicationError::worker_error)?;

    info!("Entering REPL");

    let stdin = io::stdin();
    let stdin = stdin.lock();

    print!(":> ");
    io::stdout().flush().unwrap();

    for line in stdin.lines() {
        let line = line.unwrap();

        index
            .read(|reader, schema| {
                let searcher = reader.searcher();
                let fields = schema.values().cloned().collect();
                let query_parser = QueryParser::for_index(searcher.index(), fields);

                match query_parser.parse_query(&line) {
                    Ok(query) => {
                        let top_docs: Vec<_> =
                            searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

                        for (score, doc_address) in top_docs {
                            let retrieved_doc = searcher.doc(doc_address).unwrap();

                            println!("{} => {}", score, searcher.schema().to_json(&retrieved_doc));
                        }
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }

                Ok(())
            })
            .unwrap();

        print!(":> ");
        io::stdout().flush().unwrap();
    }

    print!("");

    Ok(())
}
