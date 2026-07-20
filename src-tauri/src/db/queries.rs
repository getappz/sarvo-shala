use crate::db::Db;
use crate::error::AppError;
use rusqlite::OptionalExtension;

pub fn set_setting(db: &Db, key: &str, value: &str) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, value),
        )
    })?;
    Ok(())
}

pub fn get_setting(db: &Db, key: &str) -> Result<Option<String>, AppError> {
    let v: Option<String> = db.with(|c| {
        c.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |r| r.get(0),
        )
        .optional()
    })?;
    Ok(v)
}
