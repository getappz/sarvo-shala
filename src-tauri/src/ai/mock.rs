use crate::ai::{GenRequest, InferenceBackend};
use crate::error::AppError;
use async_trait::async_trait;
use std::time::Duration;

/// Deterministic stand-in for a real LLM. Builds a Socratic reply from the
/// directive in `system` and the first retrieved snippet, then streams it
/// word-by-word. Identical `GenRequest`s always produce identical output.
pub struct MockBackend;

#[async_trait]
impl InferenceBackend for MockBackend {
    async fn generate(
        &self,
        req: GenRequest,
        on_token: &(dyn Fn(String) + Send + Sync),
    ) -> Result<String, AppError> {
        let first_snippet = req.context.first().cloned().unwrap_or_default();
        let first_sentence = first_snippet
            .split('.')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        let reply = if first_sentence.is_empty() {
            "Let's think about this together. What do you already know about this topic?".to_string()
        } else {
            format!(
                "Let's think about this together. {} What do you already know about this topic?",
                first_sentence
            )
        };

        let words: Vec<&str> = reply.split(' ').collect();
        let mut full = String::new();
        for (i, w) in words.iter().enumerate() {
            let tok = if i + 1 < words.len() {
                format!("{w} ")
            } else {
                (*w).to_string()
            };
            on_token(tok.clone());
            full.push_str(&tok);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(full)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn mock_backend_streams_deterministically() {
        let req = GenRequest {
            system: "Ask one open question that makes the student state what they already know.".to_string(),
            user: "what is pressure".to_string(),
            context: vec!["Pressure tells us how concentrated a force is. It is force divided by area.".to_string()],
        };
        let c1 = AtomicUsize::new(0);
        let c1ref = &c1;
        let t1 = MockBackend
            .generate(req.clone(), &| _| { c1ref.fetch_add(1, Ordering::SeqCst); })
            .await
            .unwrap();
        let c2 = AtomicUsize::new(0);
        let c2ref = &c2;
        let t2 = MockBackend
            .generate(req.clone(), &| _| { c2ref.fetch_add(1, Ordering::SeqCst); })
            .await
            .unwrap();
        assert_eq!(t1, t2);
        assert!(c1.load(Ordering::SeqCst) >= 5);
        assert!(c2.load(Ordering::SeqCst) >= 5);
    }
}
