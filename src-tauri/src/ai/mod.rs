use crate::error::AppError;

/// A generation request handed to an [`InferenceBackend`].
pub struct GenRequest {
    pub system: String,
    pub user: String,
    pub context: Vec<String>,
}

#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Stream response chunks; call `on_token` per chunk; return full text.
    async fn generate(
        &self,
        req: GenRequest,
        on_token: &(dyn Fn(String) + Send + Sync),
    ) -> Result<String, AppError>;
}

pub trait Retriever: Send + Sync {
    fn retrieve(
        &self,
        query: &str,
        chapter_id: i64,
        k: usize,
    ) -> Result<Vec<Snippet>, AppError>;
}

pub struct Snippet {
    pub id: i64,
    pub body: String,
    pub score: f64,
}

/// Placeholder seam — no methods used in MVP-0.
pub trait SpeechEngine: Send + Sync {}

pub mod classify;
pub mod retriever;
