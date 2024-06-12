use crate::types::pagination::{self, Pagination};
use crate::types::{NewQuestion, Question};
use crate::{check_profanity, Store};
use std::collections::HashMap;
use tracing::{event, instrument, Level};
use warp::http::StatusCode;

// TODO: get a single question with an id param
#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "eroteme", Level::INFO, "querying questions");

    let pagination = if params.is_empty() {
        Pagination::default()
    } else {
        event!(Level::INFO, pagination = true);
        pagination::extract_pagination(&params)?
    };

    match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// # Errors
///
/// Will return `Err` if the Warp filter fails to match the route
pub async fn add_question(
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    let title = match check_profanity(new_question.title).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let content = match check_profanity(new_question.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let question = NewQuestion {
        title,
        content,
        tags: new_question.tags,
    };

    match store.add_questions(question).await {
        Ok(question) => Ok(warp::reply::json(&question)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// # Errors
///
/// Will return `Err` if the Warp filter fails to match the route
pub async fn update_question(
    id: i32,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let title = match check_profanity(question.title).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let content = match check_profanity(question.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let question = Question {
        id: question.id,
        title,
        content,
        tags: question.tags,
    };

    match store.update_question(question, id).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// # Errors
///
/// Will return `Err` if the Warp filter fails to match the route
pub async fn delete_question(id: i32, store: Store) -> Result<impl warp::Reply, warp::Rejection> {
    match store.delete_question(id).await {
        Ok(_) => Ok(warp::reply::with_status(
            format!("Question {id} deleted"),
            StatusCode::OK,
        )),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
