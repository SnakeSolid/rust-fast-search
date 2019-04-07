use super::error::WorkerError;
use super::error::WorkerResult;
use crate::config::ConfigRef;
use crate::config::DataType;
use crate::config::FieldConfig;
use crate::index::TextIndexRef;
use fallible_iterator::FallibleIterator;
use postgres::rows::LazyRows;
use postgres::rows::Row;
use postgres::types::FromSql;
use postgres::Connection;
use postgres::TlsMode;
use std::collections::HashMap;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;
use tantivy::schema::Field;
use tantivy::Document;
use tantivy::IndexWriter;

#[derive(Debug)]
pub struct PostgresWorker {
    config: ConfigRef,
    index: TextIndexRef,
    interval: Duration,
}

impl PostgresWorker {
    pub fn new(config: &ConfigRef, index: &TextIndexRef) -> WorkerResult<Self> {
        let config = config.clone();
        let index = index.clone();
        let interval = Duration::from_secs(config.interval());

        Ok(PostgresWorker {
            config,
            index,
            interval,
        })
    }

    pub fn run(mut self) -> ! {
        info!("Running PostgreSQL worker");

        loop {
            if let Err(err) = self.update() {
                warn!("Failed to update index - {}", err);
            }

            info!("Sleep for {} seconds", self.interval.as_secs());

            sleep(self.interval)
        }
    }

    fn update(&mut self) -> WorkerResult<()> {
        let mut state = self.read_state()?;
        let key_name = self.config.datasource().key();
        let config_schema = self.config.schema();
        let connection = self.connect()?;
        let query = self.config.datasource().query();
        let statement = connection.prepare(query).unwrap();
        let transaction = connection.transaction().unwrap();
        let rows = statement
            .lazy_query(&transaction, &[&state.last_key], 1_000)
            .map_err(WorkerError::query_error)?;
        let last_key = self
            .index
            .write(|writer, schema| read_rows(writer, schema, key_name, config_schema, rows))
            .map_err(WorkerError::text_index_error)??;

        if let Some(last_key) = last_key {
            state.last_key = last_key;

            self.write_state(&state)?;
        }

        Ok(())
    }
    fn write_state(&self, state: &State) -> WorkerResult<()> {
        let path = self.config.state_file();
        let writer = File::create(path).map_err(WorkerError::io_error)?;

        serde_yaml::to_writer(writer, state).map_err(WorkerError::yaml_error)?;

        Ok(())
    }

    fn read_state(&self) -> WorkerResult<State> {
        let path = self.config.state_file();

        if path.exists() {
            let reader = File::open(path).map_err(WorkerError::io_error)?;
            let state = serde_yaml::from_reader(reader).map_err(WorkerError::yaml_error)?;

            Ok(state)
        } else {
            Ok(State::default())
        }
    }

    fn connect(&self) -> WorkerResult<Connection> {
        let url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.config.datasource().user(),
            self.config.datasource().password(),
            self.config.datasource().host(),
            self.config.datasource().port(),
            self.config.datasource().database(),
        );

        Connection::connect(url, TlsMode::None).map_err(WorkerError::connection_error)
    }
}

fn read_rows(
    writer: &mut IndexWriter,
    schema: &HashMap<String, Field>,
    key_name: &str,
    config_schema: &[FieldConfig],
    mut rows: LazyRows,
) -> WorkerResult<Option<i64>> {
    let mut last_key = None;

    while let Some(row) = rows.next().map_err(WorkerError::stream_error)? {
        let mut document = Document::new();

        for field_config in config_schema {
            let name = field_config.name();
            let field = schema
                .get(name)
                .cloned()
                .ok_or_else(|| WorkerError::field_not_found(name))?;
            let column = field_config.column();

            match field_config.data_type() {
                DataType::Int { .. } => {
                    if let Some(value) = get_value(&row, column)? {
                        document.add_i64(field, value);

                        if column == key_name {
                            last_key = Some(value);
                        }
                    }
                }
                DataType::UInt { .. } => {
                    if let Some(value) = get_value::<u32>(&row, column)? {
                        document.add_u64(field, u64::from(value));
                    }
                }
                DataType::Text => {
                    if let Some(value) = get_value::<String>(&row, column)? {
                        document.add_text(field, value.trim());
                    }
                }
            }
        }

        writer.add_document(document);
    }

    Ok(last_key)
}

fn get_value<T>(row: &Row, column: &str) -> WorkerResult<Option<T>>
where
    T: FromSql,
{
    match row.get(column) {
        Some(value) => Ok(value),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct State {
    last_key: i64,
}

impl Default for State {
    fn default() -> Self {
        State {
            last_key: i64::min_value(),
        }
    }
}
