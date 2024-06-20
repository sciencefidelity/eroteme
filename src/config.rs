use clap::Parser;
use std::env;

/// Eroteme web service API
#[derive(Parser, Debug, PartialEq, Eq)]
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
    /// # Errors
    ///
    /// Will return `Err` if the port variable cannot be parsed.
    ///
    /// # Panics
    ///
    /// Will panic if env vars are not found in the environment.
    pub fn new() -> Result<Self, handle_errors::Error> {
        let config = Self::parse();

        assert!(
            env::var("BAD_WORDS_API_KEY").is_ok(),
            "missing BadWords API key"
        );

        assert!(env::var("PASETO_KEY").is_ok(), "missing Paseto key");

        let port = env::var("PORT")
            .ok()
            .map_or(Ok(config.port), |val| val.parse::<u16>())
            .map_err(handle_errors::Error::ParseError)?;

        let db_user = env::var("POSTGRES_USER").unwrap_or_else(|_| config.db_user.clone());
        let db_password =
            env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| config.db_password.clone());
        let db_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| config.db_host.clone());
        let db_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| config.db_port.to_string());
        let db_name = env::var("POSTGRES_DB").unwrap_or_else(|_| config.db_name.clone());

        Ok(Self {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(handle_errors::Error::ParseError)?,
            db_name,
        })
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    fn set_env() {
        unsafe {
            env::set_var("BAD_WORDS_API_KEY", "API_KEY");
            env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
            env::set_var("POSTGRES_USER", "user");
            env::set_var("POSTGRES_PASSWORD", "pass");
            env::set_var("POSTGRES_HOST", "localhost");
            env::set_var("POSTGRES_PORT", "5432");
            env::set_var("POSTGRES_DB", "eroteme");
        }
    }

    #[test]
    fn unset_set_api_key() {
        // TODO: these have default values
        // let result = std::panic::catch_unwind(|| Config::new());
        // assert!(result.is_err());

        set_env();

        let expected = Config {
            log_level: "warn".to_owned(),
            port: 8080,
            db_user: "user".to_owned(),
            db_password: "pass".to_owned(),
            db_host: "localhost".to_owned(),
            db_port: 5432,
            db_name: "eroteme".to_owned(),
        };

        let config = Config::new().unwrap();

        assert_eq!(config, expected);
    }
}
