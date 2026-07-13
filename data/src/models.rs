use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One entry from argonaut.json — either a "class" (no classId) or an
/// "object" (has classId), per the existing parse_draw convention.
/// The full original JSON is kept as an opaque blob in `data`, so nothing
/// is lost even if the schema has fields we don't explicitly model here.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct ArgonautEntry {
    #[primary_key]
    pub key: String,

    #[secondary_key(optional)]
    pub class_id: Option<String>,

    #[secondary_key(optional)]
    pub super_class_id: Option<String>,

    /// The full original JSON object, stored as-is.
    pub data: Value,
}
