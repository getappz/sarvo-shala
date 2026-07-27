use crate::schema::SubjectFile;

const MODEL: &str = "claude-sonnet-5";
const API_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct GenerateArgs {
    pub text: String,
    pub subject: String,
    pub grade: i64,
    pub chapter_title: String,
    pub source_authority: String,
    pub use_claude: bool,
}

fn build_prompt(args: &GenerateArgs) -> String {
    let schema = r#"{"subject":"...","subject_hi":"...","grade":0,"chapters":[{"id":"...","title":"...","title_hi":"","summary":"...","concepts":[{"id":"...","name":"...","key_formula":"","common_error":"","exam_weight":3,"prereq_ids":[],"text":[""],"questions":[{"bloom_level":"remember|understand|apply|analyze|evaluate","difficulty_layer":1|2|3,"prompt":"","explanation":"","options":[{"body":"","is_correct":true|false}]}],"source_authority":"...","generation_method":"llm_generated","review_status":"pending"}]}]}"#;
    format!(
        r#"You are an expert curriculum designer for Indian school education (NCERT/CBSE).
Your task: produce original, syllabus-aligned educational content for one chapter.

Critical legal constraint: You may NOT quote or closely paraphrase any copyrighted textbook material. Every explanation, concept description, and question must be your own original writing.

Subject: {subject}
Grade: {grade}
Chapter title: {chapter_title}
Source authority: {source_authority}

Source text (reference only, do not copy):
---
{text}
---

Produce a JSON object conforming to this schema:

{schema}

Fill "..." placeholders with real content. Set grade to {grade} and source_authority to "{source_authority}".
Generate 3 concepts per chapter, 3 questions per concept spread across difficulty layers (1-3).
Use dotted IDs like "grade_subject.chapter.concept". Exactly one correct option per question.
Respond with ONLY the JSON object, no extra text or markdown fences."#,
        subject = args.subject,
        grade = args.grade,
        chapter_title = args.chapter_title,
        source_authority = args.source_authority,
        text = args.text,
        schema = schema,
    )
}

pub fn generate(args: &GenerateArgs) -> anyhow::Result<SubjectFile> {
    let prompt = build_prompt(args);

    let raw = if args.use_claude {
        call_claude(&prompt)?
    } else {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!("ANTHROPIC_API_KEY not set; use --use-claude to run via claude CLI")
        })?;
        call_anthropic_api(&prompt, &api_key)?
    };

    let json_str = extract_json(&raw);
    let subject: SubjectFile = serde_json::from_str(json_str)?;
    Ok(subject)
}

fn call_anthropic_api(prompt: &str, api_key: &str) -> anyhow::Result<String> {
    let body = serde_json::json!({
        "model": MODEL,
        "max_tokens": 8192,
        "messages": [{"role": "user", "content": prompt}]
    });

    let resp = ureq::post(API_URL)
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .set("Content-Type", "application/json")
        .send_json(&body)?
        .into_string()?;

    let parsed: serde_json::Value = serde_json::from_str(&resp)?;
    let content = parsed["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|block| block["text"].as_str())
        .ok_or_else(|| anyhow::anyhow!("unexpected API response format: {resp}"))?;
    Ok(content.to_string())
}

fn call_claude(prompt: &str) -> anyhow::Result<String> {
    use std::io::Write;

    let mut child = std::process::Command::new("claude")
        .arg("-p")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn claude CLI: {e}"))?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())?;
        stdin.flush()?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "claude CLI exited with {}: stderr: {}",
            output.status,
            stderr.trim(),
        );
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn extract_json(text: &str) -> &str {
    if let Some(start) = text.find("```json") {
        let body = &text[start + 7..];
        if let Some(end) = body.find("```") {
            return body[..end].trim();
        }
    }
    if let Some(start) = text.find('{') {
        let mut depth = 0;
        for (i, ch) in text[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return text[start..start + i + 1].trim();
                    }
                }
                _ => {}
            }
        }
    }
    text.trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_prompt_contains_subject_and_chapter() {
        let args = GenerateArgs {
            text: "Sample extracted text about forces.".to_string(),
            subject: "Science".to_string(),
            grade: 8,
            chapter_title: "Force and Pressure".to_string(),
            source_authority: "CBSE Class 8 Science".to_string(),
            use_claude: false,
        };
        let prompt = build_prompt(&args);
        assert!(prompt.contains("Science"));
        assert!(prompt.contains("Force and Pressure"));
        assert!(prompt.contains("Grade: 8") || prompt.contains("\"grade\":8"));
        assert!(prompt.contains("CBSE Class 8 Science"));
        assert!(prompt.contains("llm_generated"));
    }

    #[test]
    fn extract_json_from_markdown_fence() {
        let input = "Here is the result:\n```json\n{\"key\": \"value\"}\n```\n";
        assert_eq!(extract_json(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn extract_json_from_plain() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(extract_json(input), "{\"key\": \"value\"}");
    }
}
