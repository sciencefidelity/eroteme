use clap::Parser;
use eroteme::{routes, store::Store};
use serde::Deserialize;
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{http::Method, Filter};

/// Eroteme web service API
#[derive(Parser, Debug, Default, Deserialize, PartialEq)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which errors we want to log (info, warn, or error)
    #[clap(short, long, default_value = "warn")]
    log_level: String,
    /// URL for the postgres database
    #[clap(long, default_value = "postgres")]
    db_user: String,
    /// Database password
    #[clap(long, default_value = "password")]
    db_password: String,
    /// Database hostname
    #[clap(long, default_value = "localhost")]
    db_host: String,
    /// Port number for the postgres database
    #[clap(long, default_value = "5432")]
    db_port: u16,
    /// Database name
    #[clap(long, default_value = "eroteme")]
    db_name: String,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();

    assert!(
        env::var("BAD_WORDS_API_KEY").is_ok(),
        "missing BadWords API key"
    );

    assert!(env::var("PASETO_KEY").is_ok(), "missing Paseto key");

    let port = env::var("PORT")
        .ok()
        .map_or(Ok(8080), |val| val.parse::<u16>())
        .map_err(handle_errors::Error::ParseError)?;

    let args = Args::parse();

    let log_filter = env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},eroteme={},warp={}",
            args.log_level, args.log_level, args.log_level
        )
    });

    let store = Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        args.db_user, args.db_password, args.db_host, args.db_port, args.db_name
    ))
    .await
    .map_err(handle_errors::Error::DatabaseQueryError)?;

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError)?;

    let store_filter = warp::any().map(move || store.clone());

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::get_questions);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::delete_question);

    // TODO: change route to `/questions/:question_id/answers`
    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::add_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::login);

    let routes = get_questions
        .or(update_question)
        .or(add_question)
        .or(delete_question)
        .or(add_answer)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
