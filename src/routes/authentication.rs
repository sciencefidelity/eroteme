use crate::{store::Store, types::Account};
use argon2::{self, Config};
use rand::Rng;
use warp::http::StatusCode;

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
        Ok(_) => Ok(warp::reply::with_status("account added", StatusCode::OK)),
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
