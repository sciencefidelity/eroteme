use eroteme::{oneshot, setup_store, Config};
use futures_util::FutureExt;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use std::process::Command;
use std::{panic, process};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Question {
    title: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QuestionAnswer {
    id: i32,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Token(String);

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();
    let config = Config::new().expect("config can't be set");

    let s = Command::new("sqlx")
        .arg("database")
        .arg("drop")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
        ))
        .arg("-y")
        .output()
        .expect("sqlx command failed to start");

    io::stdout()
        .write_all(&s.stderr)
        .expect("failed to write to stdout");

    let s = Command::new("sqlx")
        .arg("database")
        .arg("create")
        .arg("--database-url")
        .arg(format!(
            "postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
        ))
        .output()
        .expect("sqlx command failed to start");

    io::stdout()
        .write_all(&s.stderr)
        .expect("failed to write to stdout");

    let store = setup_store(&config).await?;

    let handler = oneshot(store).await;

    let user = User {
        email: "test&email.com".to_owned(),
        password: "password".to_owned(),
    };

    let token: Token;

    print!("Running register_new_user...");

    let result = panic::AssertUnwindSafe(register_new_user(&user))
        .catch_unwind()
        .await;

    if matches!(result, Ok(())) {
        println!("ok");
    } else {
        let _ = handler.sender.send(1);
        process::exit(1);
    }

    print!("running login...");

    if let Ok(t) = panic::AssertUnwindSafe(login(user)).catch_unwind().await {
        token = t;
        println!("ok");
    } else {
        let _ = handler.sender.send(1);
        process::exit(1);
    }

    print!("running post_question...");

    if matches!(
        panic::AssertUnwindSafe(post_question(token))
            .catch_unwind()
            .await,
        Ok(())
    ) {
        println!("ok");
    } else {
        let _ = handler.sender.send(1);
        process::exit(1);
    }

    let _ = handler.sender.send(1);

    Ok(())
}

async fn register_new_user(user: &User) {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/registration")
        .json(&user)
        .send()
        .await
        .expect("register_new_user failed")
        .json::<Value>()
        .await
        .expect("register_new_user failed");

    assert_eq!(res, "account added".to_owned());
}

async fn login(user: User) -> Token {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/login")
        .json(&user)
        .send()
        .await
        .expect("login failed");

    res.json::<Token>().await.expect("login failed")
}

async fn post_question(token: Token) {
    let q = Question {
        title: "First Question".to_owned(),
        content: "How can I test?".to_owned(),
    };

    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/questions")
        .header(header::AUTHORIZATION, token.0)
        .json(&q)
        .send()
        .await
        .expect("post_question failed")
        .json::<QuestionAnswer>()
        .await
        .expect("post_question failed");

    assert_eq!(res.title, q.title);
}
