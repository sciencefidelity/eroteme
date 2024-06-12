use crate::types::NewAnswer;
use crate::Store;

use warp::http::StatusCode;

/// # Errors
///
/// Will return `Err` if the Warp filter fails to match the route
pub async fn add_answer(
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.add_answer(new_answer).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
