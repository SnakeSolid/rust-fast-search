mod error;
mod validate;

pub use self::error::ConfigError;
pub use self::error::ConfigResult;
pub use self::validate::validate;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

pub type ConfigRef = Arc<Config>;

#[derive(Debug, Deserialize)]
pub struct Config {
    index_path: PathBuf,
    state_file: PathBuf,
    interval: u64,
    datasource: DatasourceConfig,
    schema: Vec<FieldConfig>,
}

impl Config {
    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    pub fn state_file(&self) -> &Path {
        &self.state_file
    }

    pub fn interval(&self) -> u64 {
        self.interval
    }

    pub fn datasource(&self) -> &DatasourceConfig {
        &self.datasource
    }

    pub fn schema(&self) -> &[FieldConfig] {
        &self.schema
    }
}

#[derive(Debug, Deserialize)]
pub struct DatasourceConfig {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
    key: String,
    query: String,
}

impl DatasourceConfig {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

#[derive(Debug, Deserialize)]
pub struct FieldConfig {
    name: String,
    column: String,
    display: String,
    data_type: DataType,
}

impl FieldConfig {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn column(&self) -> &str {
        &self.column
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum DataType {
    Int { indexed: bool },
    UInt { indexed: bool },
    Text,
}

pub fn load<P>(path: P) -> ConfigResult<ConfigRef>
where
    P: AsRef<Path>,
{
    let reader = File::open(path).map_err(ConfigError::io_error)?;
    let config = serde_yaml::from_reader(reader).map_err(ConfigError::yaml_error)?;

    validate::validate(&config)?;

    Ok(Arc::new(config))
}
