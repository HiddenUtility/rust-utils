use crate::error::JsonError;
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::Path;

pub fn to_json<T>(value: &T) -> Result<String, JsonError>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(JsonError::from)
}

pub fn from_json<T>(json: &str) -> Result<T, JsonError>
where
    T: DeserializeOwned,
{
    serde_json::from_str(json).map_err(JsonError::from)
}

pub fn to_json_pretty<T>(value: &T) -> Result<String, JsonError>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(JsonError::from)
}

pub fn read_json_file<T, P>(path: P) -> Result<T, JsonError>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let content = fs::read_to_string(path)?;
    from_json(&content)
}

pub fn write_json_file<T, P>(path: P, value: &T) -> Result<(), JsonError>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let content = to_json(value)?;
    fs::write(path, content)?;
    Ok(())
}
