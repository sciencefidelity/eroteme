use crate::store::Store;
use crate::types::{Account, AccountId, Session};
use argon2::{self, Config};
use chrono::prelude::*;
use rand::Rng;
use std::{env, future};
use warp::Filter;

/// # Errors
///
/// Will return `Err` if creating the account in the database fails.
pub async fn register(store: Store, account: Account) -> Result<impl warp::Reply, warp::Rejection> {
    let hashed_password = hash_password(account.password.as_bytes());

    let account = Account {
        id: account.id,
        email: account.email,
        password: hashed_password,
    };

    match store.add_account(account).await {
        Ok(_) => Ok(warp::reply::json(&"account added".to_string())),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// # Panics
///
/// Will panic if `hash_encoded` fails to hash password.
#[must_use]
pub fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).expect("failed to hash password")
}

/// # Errors
///
/// Will return `Err` if the wrong username/password combination is used or if the `argon2`
/// library fails to verify the hashed password.
///
/// # Panics
///
/// Will panic if account id cannot be found in the database.
pub async fn login(store: Store, login: Account) -> Result<impl warp::Reply, warp::Rejection> {
    match store.get_account(login.email).await {
        Ok(account) => match verify_password(&account.password, login.password.as_bytes()) {
            Ok(verified) => {
                if verified {
                    Ok(warp::reply::json(&issue_token(
                        &account.id.expect("id not found"),
                    )))
                } else {
                    Err(warp::reject::custom(handle_errors::Error::WrongPassword))
                }
            }
            Err(e) => Err(warp::reject::custom(
                handle_errors::Error::ArgonLibraryError(e),
            )),
        },
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// # Errors
///
/// Will return `Err` if decryption of token fails
///
/// # Panics
///
/// Will panic if `PASETO_KEY` env var is not set.
pub fn verify_token(token: &str) -> Result<Session, handle_errors::Error> {
    let key = env::var("PASETO_KEY").expect("PASETO_KEY env var not set");

    let token = paseto::tokens::validate_local_token(
        token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
    .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token).map_err(|_| handle_errors::Error::CannotDecryptToken)
}

fn verify_password(hash: &str, password: &[u8]) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

/// # Panics
///
/// Will panic if `PASETO_KEY` env var is not set.
fn issue_token(account_id: &AccountId) -> String {
    let key = env::var("PASETO_KEY").expect("PASETO_KEY env var not set");

    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(key.as_bytes()))
        .set_expiration(&dt)
        .set_claim("account_id", serde_json::json!(account_id))
        .build()
        .expect("failed to construct paseto token with builder")
}

#[must_use]
pub fn auth() -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let Ok(token) = verify_token(&token) else {
            return future::ready(Err(warp::reject::custom(
                handle_errors::Error::Unauthorized,
            )));
        };

        future::ready(Ok(token))
    })
}

#[cfg(test)]
mod authentication_tests {
    use super::{auth, env, issue_token, AccountId};

    #[tokio::test]
    async fn post_questions_auth() {
        unsafe {
            env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        }
        let token = issue_token(&AccountId(3));

        let filter = auth();

        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);

        assert_eq!(res.await.unwrap().account_id, AccountId(3));
    }
}
