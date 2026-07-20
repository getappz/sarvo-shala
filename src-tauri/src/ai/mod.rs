use crate::error::AppError;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// A generation request handed to an [`InferenceBackend`].
///
/// `system`/`user` mirror the fields a real LLM backend will template; the
/// MVP-0 mock only consumes `state`/`context`, so the contract fields are
/// intentionally part of the API and not read yet.
#[derive(Clone)]
#[allow(dead_code)]
pub struct GenRequest {
    pub system: String,
    pub user: String,
    pub state: String,
    pub context: Vec<String>,
}

#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Stream response chunks; call `on_token` per chunk; return full text.
    /// `cancel` can be tripped to abort a slow generation (e.g. when the user
    /// navigates away) without blocking the command.
    async fn generate(
        &self,
        req: GenRequest,
        cancel: &Arc<AtomicBool>,
        on_token: &(dyn Fn(String) + Send + Sync),
    ) -> Result<String, AppError>;
}

/// Deliberate prompt boundary for student free-text. When a real LLM backend
/// templates `user` into a system prompt, this keeps the student's words
/// fenced off from the instruction text (preventing injection / delimiter
/// confusion). MockBackend ignores it, but the contract exists from MVP-0.
pub fn safe_user_prompt(text: &str) -> String {
    // Strip control characters and fence the content so it cannot be mistaken
    // for instruction syntax by a downstream template.
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();
    format!("[STUDENT_MESSAGE]\n{}\n[/STUDENT_MESSAGE]", cleaned.trim())
}

pub trait Retriever: Send + Sync {
    fn retrieve(&self, query: &str, chapter_id: i64, k: usize) -> Result<Vec<Snippet>, AppError>;
}

/// A retrieved curriculum snippet. `id`/`score` are populated for future
/// ranking backends; the MVP-0 mock only reads `body`.
#[allow(dead_code)]
pub struct Snippet {
    pub id: i64,
    pub body: String,
    pub score: f64,
}

pub mod classify;
pub mod mock;
pub mod retriever;
pub mod socratic;
