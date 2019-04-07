mod error;

pub use self::error::TextIndexError;
pub use self::error::TextIndexResult;

use crate::config::ConfigRef;
use crate::config::DataType;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::sync::Arc;
use std::sync::Mutex;
use tantivy::directory::MmapDirectory;
use tantivy::schema::Field;
use tantivy::schema::SchemaBuilder;
use tantivy::schema::INDEXED;
use tantivy::schema::STORED;
use tantivy::schema::TEXT;
use tantivy::Index;
use tantivy::IndexReader;
use tantivy::IndexWriter;
use tantivy::ReloadPolicy;
use tantivy::Result as TantivyResult;

#[derive(Debug, Clone)]
pub struct TextIndexRef {
    inner: Arc<Mutex<TextIndex>>,
}

impl TextIndexRef {
    pub fn new(config: &ConfigRef, new_index: bool) -> TextIndexResult<Self> {
        info!("Preparing index...");

        let text_index = TextIndex::new(config, new_index)?;

        Ok(TextIndexRef {
            inner: Arc::new(Mutex::new(text_index)),
        })
    }

    pub fn read<F, T>(&self, callback: F) -> TextIndexResult<T>
    where
        F: FnOnce(&IndexReader, &HashMap<String, Field>) -> TantivyResult<T>,
    {
        self.inner
            .lock()
            .map_err(TextIndexError::poison_error)?
            .read(callback)
    }

    pub fn write<F, T, E>(&mut self, callback: F) -> TextIndexResult<Result<T, E>>
    where
        F: FnOnce(&mut IndexWriter, &HashMap<String, Field>) -> Result<T, E>,
    {
        self.inner
            .lock()
            .map_err(TextIndexError::poison_error)?
            .write(callback)
    }
}

pub struct TextIndex {
    fields: HashMap<String, Field>,
    reader: IndexReader,
    writer: IndexWriter,
}

impl TextIndex {
    fn new(config: &ConfigRef, new_index: bool) -> TextIndexResult<Self> {
        let mut schema_builder = SchemaBuilder::default();
        let mut fields = HashMap::new();

        for field_config in config.schema() {
            let name = field_config.name().to_string();
            let field = match field_config.data_type() {
                DataType::Int { indexed: true } => {
                    schema_builder.add_i64_field(&name, INDEXED | STORED)
                }
                DataType::Int { indexed: false } => schema_builder.add_i64_field(&name, STORED),
                DataType::UInt { indexed: true } => {
                    schema_builder.add_u64_field(&name, INDEXED | STORED)
                }
                DataType::UInt { indexed: false } => schema_builder.add_u64_field(&name, STORED),
                DataType::Text => schema_builder.add_text_field(&name, TEXT | STORED),
            };

            fields.insert(name, field);
        }

        let schema = schema_builder.build();
        let directory = MmapDirectory::open(config.index_path())
            .map_err(TextIndexError::open_directory_error)?;
        let index = if new_index {
            Index::create(directory, schema.clone())
        } else {
            Index::open_or_create(directory, schema.clone())
        };
        let index = index.map_err(TextIndexError::tantivy_error)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .num_searchers(1)
            .try_into()
            .map_err(TextIndexError::tantivy_error)?;
        let writer = index
            .writer(0x0040_0000)
            .map_err(TextIndexError::tantivy_error)?;

        Ok(TextIndex {
            fields,
            reader,
            writer,
        })
    }

    fn read<F, T>(&self, callback: F) -> TextIndexResult<T>
    where
        F: FnOnce(&IndexReader, &HashMap<String, Field>) -> TantivyResult<T>,
    {
        callback(&self.reader, &self.fields).map_err(TextIndexError::read_error)
    }

    fn write<F, T, E>(&mut self, callback: F) -> TextIndexResult<Result<T, E>>
    where
        F: FnOnce(&mut IndexWriter, &HashMap<String, Field>) -> Result<T, E>,
    {
        let result = callback(&mut self.writer, &self.fields);

        match result {
            Ok(_) => {
                self.writer
                    .commit()
                    .map_err(TextIndexError::tantivy_error)?;
            }
            Err(_) => {
                self.writer
                    .rollback()
                    .map_err(TextIndexError::tantivy_error)?;
            }
        }

        Ok(result)
    }
}

impl Debug for TextIndex {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "TextIndex {{ fields: {:?}, ... }}", self.fields)
    }
}
