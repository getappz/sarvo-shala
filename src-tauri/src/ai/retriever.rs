use crate::ai::{Retriever, Snippet};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;

/// Keyword retriever over a chapter's `chapter_text` rows.
pub struct KeywordRetriever {
    db: Db,
}

impl KeywordRetriever {
    pub fn new(db: Db) -> Self {
        KeywordRetriever { db }
    }
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_string())
        .collect()
}

impl Retriever for KeywordRetriever {
    fn retrieve(&self, query: &str, chapter_id: i64, k: usize) -> Result<Vec<Snippet>, AppError> {
        let tokens = tokenize(query);
        let rows = queries::chapter_text_rows(&self.db, chapter_id)?;

        let mut scored: Vec<(i64, String, i64, f64)> = rows
            .into_iter()
            .map(|(id, body, ord)| {
                let lower = body.to_lowercase();
                let score = tokens.iter().filter(|t| lower.contains(t.as_str())).count() as f64;
                (id, body, ord, score)
            })
            .collect();

        let has_hits = scored.iter().any(|s| s.3 > 0.0);

        if has_hits {
            scored.retain(|s| s.3 > 0.0);
            scored.sort_by(|a, b| {
                b.3.partial_cmp(&a.3)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.2.cmp(&b.2))
            });
        } else {
            // No keyword hits: fall back to natural order.
            scored.sort_by(|a, b| a.2.cmp(&b.2));
        }

        Ok(scored
            .into_iter()
            .take(k)
            .map(|(id, body, _, score)| Snippet { id, body, score })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::ai::Retriever;
    use crate::db::seed;
    use crate::db::Db;

    fn seeded() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("swayam_ret_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        seed::seed_if_empty(&db).unwrap();
        (db, p)
    }

    #[test]
    fn retriever_finds_pressure_snippet() {
        let (db, p) = seeded();
        let r = super::KeywordRetriever::new(db);
        let snips = r.retrieve("what is pressure force area", 1, 3).unwrap();
        assert!(!snips.is_empty());
        assert!(
            snips[0].body.to_lowercase().contains("pressure"),
            "top snippet should mention pressure, got: {}",
            snips[0].body
        );
        let _ = std::fs::remove_file(&p);
    }
}
