use crate::types::{Answer, AnswerId, QuestionId};
use crate::Store;

use std::collections::HashMap;
use warp::http::StatusCode;

pub async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: create unique ids
    // TODO: add proper error handling for missing parameters
    // TODO: check if the question_id exists
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.get("content").unwrap().to_string(),
        question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
