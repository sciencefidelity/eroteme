use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct QuestionId(pub String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
