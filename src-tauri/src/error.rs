use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad input: {0}")]
    BadInput(String),
    #[error("engine error: {0}")]
    Engine(String),
    #[error("vfs error: {0}")]
    Vfs(#[from] agentflare_store::Error),
    #[error("harness error: {0}")]
    Harness(#[from] harness::HarnessError),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
