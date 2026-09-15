//! Versioned persistence provider contracts for UI/application state.
#![forbid(unsafe_code)]
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredRecord {
    pub schema: String,
    pub version: u32,
    pub payload: String,
}
pub trait PersistenceProvider {
    fn load(&self, key: &str) -> Result<Option<StoredRecord>, String>;
    fn store(&mut self, key: &str, record: StoredRecord) -> Result<(), String>;
    fn remove(&mut self, key: &str) -> Result<(), String>;
}
#[derive(Default)]
pub struct MemoryPersistence {
    records: BTreeMap<String, StoredRecord>,
}
impl PersistenceProvider for MemoryPersistence {
    fn load(&self, key: &str) -> Result<Option<StoredRecord>, String> {
        Ok(self.records.get(key).cloned())
    }
    fn store(&mut self, key: &str, record: StoredRecord) -> Result<(), String> {
        self.records.insert(key.into(), record);
        Ok(())
    }
    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.records.remove(key);
        Ok(())
    }
}
pub fn encode<T: Serialize>(schema: &str, version: u32, value: &T) -> Result<StoredRecord, String> {
    serde_json::to_string(value)
        .map(|payload| StoredRecord {
            schema: schema.into(),
            version,
            payload,
        })
        .map_err(|e| e.to_string())
}
pub fn decode<T: DeserializeOwned>(
    record: &StoredRecord,
    expected_schema: &str,
) -> Result<T, String> {
    if record.schema != expected_schema {
        return Err(format!("schema mismatch: {}", record.schema));
    }
    serde_json::from_str(&record.payload).map_err(|e| e.to_string())
}
