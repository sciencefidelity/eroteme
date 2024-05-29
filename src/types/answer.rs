use crate::types::question::QuestionId;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct AnswerId(pub i32);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewAnswer {
    pub content: String,
    pub question_id: QuestionId,
}
