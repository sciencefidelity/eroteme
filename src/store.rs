use crate::types::{Account, AccountId, Answer, AnswerId};
use crate::types::{NewAnswer, NewQuestion, Question, QuestionId};
use crate::Config;
use handle_errors::Error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Clone, Debug)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    /// # Errors
    ///
    /// Will return `Err` if the database migration fails.
    ///
    /// # Panics
    ///
    /// Will panic if fails to establish a database connection.
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        let db_pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
        {
            Ok(pool) => pool,
            Err(e) => panic!("couldn't establish DB connection: {e}"),
        };

        Ok(Self {
            connection: db_pool,
        })
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn get_questions(
        self,
        limit: Option<i32>,
        offset: i32,
    ) -> Result<Vec<Question>, Error> {
        match sqlx::query("SELECT * from questions LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Question {
                id: QuestionId(row.get("id")),
                title: row.get("title"),
                content: row.get("content"),
                tags: row.get("tags"),
            })
            .fetch_all(&self.connection)
            .await
        {
            Ok(questions) => Ok(questions),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if database query fails.
    pub async fn is_question_owner(
        &self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        match sqlx::query("SELECT * from questions where id = $1 and account_id = $2")
            .bind(question_id)
            .bind(account_id.0)
            .fetch_optional(&self.connection)
            .await
        {
            Ok(question) => Ok(question.is_some()),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn add_questions(
        self,
        new_question: NewQuestion,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        match sqlx::query(
            "INSERT INTO questions (title, Content, tags, account_id) 
            VALUES ($1, $2, $3, $4) 
            RETURNING id, title, content, tags",
        )
        .bind(new_question.title)
        .bind(new_question.content)
        .bind(new_question.tags)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(question) => Ok(question),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn update_question(
        self,
        question: Question,
        id: i32,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        match sqlx::query(
            "UPDATE questions
            SET title = $1, content = $2, tags = $3
            WHERE id = $4 AND account_id = $5
            RETURNING id, title, content, tags",
        )
        .bind(question.title)
        .bind(question.content)
        .bind(question.tags)
        .bind(id)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(question) => Ok(question),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn delete_question(
        self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        match sqlx::query(
            "DELETE FROM questions 
            WHERE id = $1 AND account_id = $2",
        )
        .bind(question_id)
        .bind(account_id.0)
        .execute(&self.connection)
        .await
        {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn add_answer(
        self,
        new_answer: NewAnswer,
        account_id: AccountId,
    ) -> Result<Answer, Error> {
        match sqlx::query(
            "INSERT INTO answers (content, corresponding_question, account_id) VALUES ($1, $2, $3)",
        )
        .bind(new_answer.content)
        .bind(new_answer.question_id.0)
        .bind(account_id.0)
        .map(|row: PgRow| Answer {
            id: AnswerId(row.get("id")),
            content: row.get("content"),
            question_id: QuestionId(row.get("question_id")),
        })
        .fetch_one(&self.connection)
        .await
        {
            Ok(answer) => Ok(answer),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the adding the account to db fails.
    ///
    /// # Panics
    ///
    /// Will panic if the adding the account to db fails.
    pub async fn add_account(self, account: Account) -> Result<bool, Error> {
        match sqlx::query("INSERT INTO accounts (email, password) VALUES ($1, $2)")
            .bind(account.email)
            .bind(account.password)
            .execute(&self.connection)
            .await
        {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error
                        .as_database_error()
                        .expect("database error")
                        .code()
                        .expect("failed to generate error code")
                        .parse::<i32>()
                        .expect("failed parse error code"),
                    db_message = error.as_database_error().expect("database error").message(),
                    constraint = error
                        .as_database_error()
                        .expect("database error")
                        .constraint()
                        .expect("failed to get error constraint"),
                );
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if the database query fails.
    pub async fn get_account(self, email: String) -> Result<Account, Error> {
        match sqlx::query("SELECT * from accounts where email = $1")
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password"),
            })
            .fetch_one(&self.connection)
            .await
        {
            Ok(account) => Ok(account),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }
}

/// # Errors
///
/// Will return `Err` if the database migration fails.
pub async fn setup(config: &Config) -> Result<Store, handle_errors::Error> {
    let store = Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
    ))
    .await
    .map_err(handle_errors::Error::DatabaseQueryError)?;

    sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(handle_errors::Error::MigrationError)?;

    let log_filter = format!(
        "handle_errors={},eroteme={},warp={}",
        config.log_level, config.log_level, config.log_level
    );

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}
