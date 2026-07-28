use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubjectFile {
    pub subject: String,
    pub subject_hi: String,
    pub grade: i64,
    pub chapters: Vec<ChapterAuthoring>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterAuthoring {
    pub id: String,
    pub title: String,
    pub title_hi: String,
    pub summary: String,
    pub concepts: Vec<ConceptAuthoring>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConceptAuthoring {
    pub id: String,
    pub name: String,
    pub key_formula: String,
    pub common_error: String,
    pub exam_weight: i64,
    pub prereq_ids: Vec<String>,
    pub text: Vec<String>,
    pub questions: Vec<QuestionAuthoring>,
    pub source_authority: String,
    pub generation_method: GenerationMethod,
    pub review_status: ReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionAuthoring {
    pub bloom_level: String,
    pub difficulty_layer: i64,
    pub prompt: String,
    pub explanation: String,
    pub options: Vec<OptionAuthoring>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptionAuthoring {
    pub body: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMethod {
    Original,
    LlmGenerated,
    UserUploaded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Reviewed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_file_round_trips_through_json() {
        let file = SubjectFile {
            subject: "Science".to_string(),
            subject_hi: "विज्ञान".to_string(),
            grade: 8,
            chapters: vec![ChapterAuthoring {
                id: "science8.crop_production".to_string(),
                title: "Crop Production".to_string(),
                title_hi: String::new(),
                summary: "How food crops are grown and managed.".to_string(),
                concepts: vec![ConceptAuthoring {
                    id: "science8.crop_production.agricultural_practices".to_string(),
                    name: "Agricultural practices".to_string(),
                    key_formula: String::new(),
                    common_error: String::new(),
                    exam_weight: 3,
                    prereq_ids: vec![],
                    text: vec!["Farmers prepare soil before sowing seeds.".to_string()],
                    questions: vec![QuestionAuthoring {
                        bloom_level: "remember".to_string(),
                        difficulty_layer: 1,
                        prompt: "What is the first step in growing a crop?".to_string(),
                        explanation: "Soil must be prepared before sowing.".to_string(),
                        options: vec![
                            OptionAuthoring {
                                body: "Preparing the soil".to_string(),
                                is_correct: true,
                            },
                            OptionAuthoring {
                                body: "Harvesting".to_string(),
                                is_correct: false,
                            },
                        ],
                    }],
                    source_authority: "CBSE Class 8 Science".to_string(),
                    generation_method: GenerationMethod::LlmGenerated,
                    review_status: ReviewStatus::Pending,
                }],
            }],
        };
        let json = serde_json::to_string_pretty(&file).unwrap();
        let round_tripped: SubjectFile = serde_json::from_str(&json).unwrap();
        assert_eq!(file, round_tripped);
    }

    #[test]
    fn generation_method_serializes_as_snake_case() {
        let json = serde_json::to_string(&GenerationMethod::LlmGenerated).unwrap();
        assert_eq!(json, "\"llm_generated\"");
    }
}
