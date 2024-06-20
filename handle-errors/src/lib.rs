use tracing::{event, instrument, Level};
use warp::filters::body::BodyDeserializeError;
use warp::filters::cors::CorsForbidden;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Rejection, Reply};

#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    ArgonLibraryError(argon2::Error),
    DatabaseQueryError(sqlx::Error),
    MigrationError(sqlx::migrate::MigrateError),
    ReqwestAPIError(reqwest::Error),
    MiddlewareReqwestError(reqwest_middleware::Error),
    ClientError(APILayerError),
    ServerError(APILayerError),
}

#[derive(Debug, Clone)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for APILayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "status: {}, message: {}", self.status, self.message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ParseError(ref err) => write!(f, "cannot parse parameter: {err}"),
            Self::MissingParameters => write!(f, "missing parameter"),
            Self::WrongPassword => write!(f, "wrong password"),
            Self::CannotDecryptToken => write!(f, "cannot decrypt token"),
            Self::Unauthorized => write!(f, "no permission to change the underlying resource"),
            Self::ArgonLibraryError(_) => write!(f, "cannot verify password"),
            Self::DatabaseQueryError(_) => write!(f, "cannot update, invalid data"),
            Self::MigrationError(_) => write!(f, "cannot migrate database"),
            Self::ReqwestAPIError(err) => write!(f, "cannot execute: {err}"),
            Self::MiddlewareReqwestError(err) => write!(f, "cannot execute: {err}"),
            Self::ClientError(err) => write!(f, "external client error: {err}"),
            Self::ServerError(err) => write!(f, "external server error: {err}"),
        }
    }
}

impl Reject for Error {}
impl Reject for APILayerError {}

const DUPLICATE_KEY: u32 = 23505;

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    #[allow(clippy::equatable_if_let)]
    if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "database query error");
        match e {
            sqlx::Error::Database(err) => {
                if err
                    .code()
                    .expect("error code not found")
                    .parse::<u32>()
                    .expect("failed to parse error code")
                    == DUPLICATE_KEY
                {
                    Ok(warp::reply::with_status(
                        "Account already exsists".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => Ok(warp::reply::with_status(
                "Cannot update data".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
        }
    } else if let Some(crate::Error::ReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{e}");
        Ok(warp::reply::with_status(
            "internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::CannotDecryptToken) = r.find() {
        event!(Level::ERROR, "not matching account id");
        Ok(warp::reply::with_status(
            "no presimmsion to change underlying resource".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "entered wrong password");
        Ok(warp::reply::with_status(
            "wrong email/password combination".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(crate::Error::MiddlewareReqwestError(e)) = r.find() {
        event!(Level::ERROR, "{e}");
        Ok(warp::reply::with_status(
            "internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ClientError(e)) = r.find() {
        event!(Level::ERROR, "{e}");
        Ok(warp::reply::with_status(
            "internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(crate::Error::ServerError(e)) = r.find() {
        event!(Level::ERROR, "{e}");
        Ok(warp::reply::with_status(
            "internal server error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "cannot deserialize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        event!(Level::WARN, "requested route was not found");
        Ok(warp::reply::with_status(
            "route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
