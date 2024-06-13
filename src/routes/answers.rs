use crate::types::{NewAnswer, Session};
use crate::{check_profanity, Store};
use warp::http::StatusCode;

/// # Errors
///
/// Will return `Err` if the Warp filter fails to match the route
pub async fn add_answer(
    session: Session,
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    let content = match check_profanity(new_answer.content).await {
        Ok(res) => res,
        Err(e) => return Err(warp::reject::custom(e)),
    };

    let answer = NewAnswer {
        content,
        question_id: new_answer.question_id,
    };

    match store.add_answer(answer, account_id).await {
        Ok(_) => Ok(warp::reply::with_status("answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
