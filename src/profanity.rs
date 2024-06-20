use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct APIResponse {
    message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BadWord {
    original: String,
    word: String,
    deviations: i64,
    info: i64,
    #[serde(rename = "replacedLen")]
    replaced_len: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BadWordsResponse {
    content: String,
    bad_words_total: i64,
    bad_words_list: Vec<BadWord>,
    censored_content: String,
}

/// # Errors
///
/// Will return `Err` if the API call responds with an error.
///
/// # Panics
///
/// Will panic if `BAD_WORDS_API_KEY` is not set.
#[allow(clippy::module_name_repetitions)]
pub async fn check_profanity(content: String) -> Result<String, handle_errors::Error> {
    let api_key = env::var("BAD_WORDS_API_KEY").expect("BadWords API key not set");
    let api_layer_url = env::var("API_LAYER_URL").expect("api layer url not set");

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let res = client
        .post(format!("{api_layer_url}/bad_words?censor_character=*"))
        .header("apikey", api_key)
        .body(content)
        .send()
        .await
        .map_err(handle_errors::Error::MiddlewareReqwestError)?;

    if !res.status().is_success() {
        if res.status().is_client_error() {
            let err = transform_error(res).await;
            return Err(handle_errors::Error::ClientError(err));
        }
        let err = transform_error(res).await;
        return Err(handle_errors::Error::ServerError(err));
    }

    match res.json::<BadWordsResponse>().await {
        Ok(res) => Ok(res.censored_content),
        Err(e) => Err(handle_errors::Error::ReqwestAPIError(e)),
    }
}

async fn transform_error(res: reqwest::Response) -> handle_errors::APILayerError {
    handle_errors::APILayerError {
        status: res.status().as_u16(),
        message: res
            .json::<APIResponse>()
            .await
            .expect("message missing in api response")
            .message,
    }
}

#[cfg(test)]
mod tests {
    use super::{check_profanity, env};
    use mock_server::{MockServer, OneshotHandler};

    #[tokio::test]
    async fn run() {
        let handler = run_mock();
        censor_profane_words().await;
        no_profane_words().await;
        let _ = handler.sender.send(1);
    }

    fn run_mock() -> OneshotHandler {
        unsafe {
            env::set_var("API_LAYER_URL", "http://127.0.0.1:3030");
            env::set_var("BAD_WORDS_API_KEY", "YES");
        }
        let socket = "127.0.0.1:3030"
            .to_owned()
            .parse()
            .expect("not a valid address");
        let mock = MockServer::new(socket);
        mock.oneshot()
    }

    async fn censor_profane_words() {
        let content = "This is a shitty sentence".to_owned();
        let censored_content = check_profanity(content).await;
        assert_eq!(censored_content.unwrap(), "this is a ****** sentence");
    }

    async fn no_profane_words() {
        let content = "This is a sentence".to_owned();
        let censored_content = check_profanity(content).await;
        assert_eq!(censored_content.unwrap(), "");
    }
}
