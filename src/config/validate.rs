use super::Config;
use super::ConfigError;
use super::ConfigResult;

use std::path::Path;

#[allow(clippy::needless_pass_by_value)]
pub fn validate(config: &Config) -> ConfigResult<()> {
    validate_number(config.interval(), "interval")?;
    validate_dir(config.index_path(), "index path")?;
    validate_file(config.state_file(), "state file")?;

    Ok(())
}

fn validate_number(value: u64, name: &str) -> ConfigResult<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(ConfigError::format(format_args!(
            "Numbers of {} must be greater than zero, but {} given",
            name, value
        )))
    }
}

fn validate_dir<P>(path: P, name: &str) -> ConfigResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        Err(ConfigError::format(format_args!(
            "File {} ({}) is not exists",
            name,
            path.display(),
        )))
    } else if !path.is_dir() {
        Err(ConfigError::format(format_args!(
            "File {} ({}) is not a directory",
            name,
            path.display()
        )))
    } else {
        Ok(())
    }
}

fn validate_file<P>(path: P, name: &str) -> ConfigResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if path.exists() && !path.is_file() {
        Err(ConfigError::format(format_args!(
            "{} directory ({}) is not a file",
            name,
            path.display()
        )))
    } else {
        Ok(())
    }
}
