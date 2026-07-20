use crate::ai::{GenRequest, InferenceBackend};
use crate::error::AppError;
use async_trait::async_trait;
use std::time::Duration;

/// Build a deterministic, Socratic-state-aware reply from the top retrieved
/// snippet's first sentence. Distinct wording per FSM state so the state
/// transition is observable in the UI (the badge and the phrasing agree).
fn template(state: &str, snippet_sentence: &str) -> String {
    let s = snippet_sentence.trim();
    if s.is_empty() {
        return match state {
            "Clarify" => "Let's make that precise. What exactly do you mean?".to_string(),
            "Confront" => "Here's a check on that claim. Does it hold in every case?".to_string(),
            "Scaffold" => {
                "One small step at a time. What is the very first thing to consider?".to_string()
            }
            "Confirm" => {
                "That's right. Now, a fresh question: what would change if the force doubled?"
                    .to_string()
            }
            _ => "Let's think about this together. What do you already know about this topic?"
                .to_string(),
        };
    }
    match state {
        "Clarify" => format!(
            "Let's make that precise. {} Can you restate your idea using the exact term?",
            s
        ),
        "Confront" => format!(
            "Here's a check on that claim. {} Does this fit what you said? Why or why not?",
            s
        ),
        "Scaffold" => format!(
            "One small step at a time. {} What happens if we change just one part of this?",
            s
        ),
        "Confirm" => format!(
            "That's right. {} Now, a fresh question: what would happen if the force doubled?",
            s
        ),
        _ => format!(
            "Let's think about this together. {} What do you already know about this topic?",
            s
        ),
    }
}

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

        let reply = template(req.state.trim(), &first_sentence);

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
            system: "Ask one open question that makes the student state what they already know."
                .to_string(),
            user: "what is pressure".to_string(),
            state: "Probe".to_string(),
            context: vec![
                "Pressure tells us how concentrated a force is. It is force divided by area."
                    .to_string(),
            ],
        };
        let c1 = AtomicUsize::new(0);
        let c1ref = &c1;
        let t1 = MockBackend
            .generate(req.clone(), &|_| {
                c1ref.fetch_add(1, Ordering::SeqCst);
            })
            .await
            .unwrap();
        let c2 = AtomicUsize::new(0);
        let c2ref = &c2;
        let t2 = MockBackend
            .generate(req.clone(), &|_| {
                c2ref.fetch_add(1, Ordering::SeqCst);
            })
            .await
            .unwrap();
        assert_eq!(t1, t2);
        assert!(c1.load(Ordering::SeqCst) >= 5);
        assert!(c2.load(Ordering::SeqCst) >= 5);
    }

    #[test]
    fn template_varies_by_state() {
        assert!(
            template("Probe", "Pressure is force over area").contains("What do you already know")
        );
        assert!(template("Scaffold", "Pressure is force over area").contains("One small step"));
        assert!(template("Confront", "Pressure is force over area").contains("check on that claim"));
        assert!(template("Confirm", "Pressure is force over area").contains("That's right"));
        // Empty snippet still yields a coherent opener per state.
        assert!(template("Scaffold", "").contains("What is the very first thing"));
    }
}
