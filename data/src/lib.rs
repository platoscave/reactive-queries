use native_db::{Builder, Database, Models};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub mod models;
pub mod queries;

pub use models::*;
pub use queries::*;

pub static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<ArgonautEntry>().unwrap();
    models
});

/// Open (or create) the on-disk database at `db_path`.
pub fn open_db(db_path: &str) -> Database<'static> {
    Builder::new()
        .create(&MODELS, db_path)
        .expect("failed to open/create database")
}

/// Parse assets/argonaut.json and insert every entry into the database.
/// Call once at startup, after `open_db`.
pub fn load_argonaut_json(db: &Database<'_>, json_path: &Path) -> Result<usize, String> {
    let text = fs::read_to_string(json_path)
        .map_err(|e| format!("failed to read {}: {e}", json_path.display()))?;

    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("failed to parse {}: {e}", json_path.display()))?;

    let array = value
        .as_array()
        .ok_or_else(|| "argonaut.json root is not an array".to_string())?;

    let rw = db
        .rw_transaction()
        .map_err(|e| format!("failed to start transaction: {e}"))?;

    let mut count = 0;
    for obj in array {
        let Some(key) = obj.get("key").and_then(|k| k.as_str()) else {
            eprintln!("warning: skipping entry with no \"key\" field: {obj}");
            continue;
        };

        let class_id = obj
            .get("classId")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let super_class_id = obj
            .get("superClassId")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let entry = ArgonautEntry {
            key: key.to_string(),
            class_id,
            super_class_id,
            data: obj.clone(),
        };

        rw.insert(entry)
            .map_err(|e| format!("failed to insert entry \"{key}\": {e}"))?;
        count += 1;
    }

    rw.commit()
        .map_err(|e| format!("failed to commit transaction: {e}"))?;

    Ok(count)
}