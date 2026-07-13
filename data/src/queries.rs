use crate::models::{ArgonautEntry, ArgonautEntryKey};
use native_db::Database;

pub fn get_by_key(db: &Database<'_>, key: &str) -> Option<ArgonautEntry> {
    let r = db.r_transaction().ok()?;
    r.get().primary(key.to_string()).ok().flatten()
}

pub fn get_by_class_id(db: &Database<'_>, class_id: &str) -> Vec<ArgonautEntry> {
    let Ok(r) = db.r_transaction() else {
        return Vec::new();
    };
    let Ok(iter) = r
        .scan()
        .secondary::<ArgonautEntry>(ArgonautEntryKey::class_id)
    else {
        return Vec::new();
    };
    let Ok(matches) = iter.start_with(class_id.to_string()) else {
        return Vec::new();
    };
    matches.filter_map(Result::ok).collect()
}

pub fn get_by_super_class_id(db: &Database<'_>, super_class_id: &str) -> Vec<ArgonautEntry> {
    let Ok(r) = db.r_transaction() else {
        return Vec::new();
    };
    let Ok(iter) = r
        .scan()
        .secondary::<ArgonautEntry>(ArgonautEntryKey::super_class_id)
    else {
        return Vec::new();
    };
    let Ok(matches) = iter.start_with(super_class_id.to_string()) else {
        return Vec::new();
    };
    matches.filter_map(Result::ok).collect()
}

pub fn get_all(db: &Database<'_>) -> Vec<ArgonautEntry> {
    let Ok(r) = db.r_transaction() else {
        return Vec::new();
    };
    let Ok(iter) = r.scan().primary::<ArgonautEntry>() else {
        return Vec::new();
    };
    let Ok(all) = iter.all() else {
        return Vec::new();
    };
    all.filter_map(Result::ok).collect()
}
