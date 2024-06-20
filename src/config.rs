use clap::Parser;
use dotenv;
use std::env;

/// Eroteme web service API
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// Which errors we want to log (info, warn, or error)
    #[clap(short, long, default_value = "warn")]
    pub log_level: String,
    /// Which port the server is listening on
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
    /// Database user
    #[clap(long, default_value = "postgres")]
    pub db_user: String,
    /// Database password
    #[clap(long, default_value = "password")]
    pub db_password: String,
    /// Database hostname
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    /// Port number for the postgres database
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    /// Database name
    #[clap(long, default_value = "eroteme")]
    pub db_name: String,
}

impl Config {
    pub fn new() -> Result<Self, handle_errors::Error> {
        dotenv::dotenv().ok();
        let config = Config::parse();

        assert!(
            env::var("BAD_WORDS_API_KEY").is_ok(),
            "missing BadWords API key"
        );

        assert!(env::var("PASETO_KEY").is_ok(), "missing Paseto key");

        let port = env::var("PORT")
            .ok()
            .map_or(Ok(config.port), |val| val.parse::<u16>())
            .map_err(handle_errors::Error::ParseError)?;

        let db_user = env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap_or(config.db_password.to_owned());
        let db_host = env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("POSTGRES_PORT").unwrap_or(config.db_port.to_string());
        let db_name = env::var("POSTGRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Self {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(|e| handle_errors::Error::ParseError(e))?,
            db_name,
        })
    }
}
