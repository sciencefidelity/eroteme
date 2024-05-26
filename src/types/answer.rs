use crate::types::question::QuestionId;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct AnswerId(pub String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}
