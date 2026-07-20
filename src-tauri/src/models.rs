use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub language: String,
    pub grade: i64,
    pub board: String,
    pub onboarded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: i64,
    pub grade: i64,
    pub name: String,
    pub name_hi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: i64,
    pub subject_id: i64,
    pub ord: i64,
    pub title: String,
    pub title_hi: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Concept {
    pub id: i64,
    pub chapter_id: i64,
    pub ord: i64,
    pub name: String,
    pub key_formula: String,
    pub common_error: String,
    pub exam_weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: i64,
    pub concept_id: i64,
    pub qtype: String,
    pub bloom_level: String,
    pub difficulty_layer: i64,
    pub prompt: String,
    pub options: Vec<QuestionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub id: i64,
    pub ord: i64,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizResult {
    pub session_id: i64,
    pub total: i64,
    pub correct: i64,
    pub score: f64,
    pub per_question: Vec<AnswerReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerReview {
    pub question_id: i64,
    pub correct: bool,
    pub correct_option_id: i64,
    pub explanation: String,
    pub trap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConceptProgress {
    pub concept_id: i64,
    pub name: String,
    pub p_known: f64,
    pub due: Option<i64>,
    pub attempts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Token { chapter_id: i64, text: String },
    StateChange { chapter_id: i64, state: String },
    Done { chapter_id: i64, message_id: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i64,
    pub chapter_id: i64,
    pub role: String,
    pub body: String,
    pub fsm_state: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TutorTurnResult {
    pub reply: String,
    pub fsm_state: String,
    pub student_msg_id: i64,
    pub tutor_msg_id: i64,
}
