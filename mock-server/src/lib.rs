use bytes::Bytes;
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::oneshot::{self, Sender};
use warp::{http, reply::Reply, Filter};

#[derive(Clone, Debug)]
pub struct MockServer {
    socket: SocketAddr,
}

pub struct OneshotHandler {
    pub sender: Sender<i32>,
}

impl MockServer {
    #[must_use]
    pub const fn new(bind_addr: SocketAddr) -> Self {
        Self { socket: bind_addr }
    }

    #[allow(clippy::unused_async)]
    async fn check_profanity((): (), content: Bytes) -> Result<impl warp::Reply, warp::Rejection> {
        let content = String::from_utf8(content.to_vec()).expect("invalid UTF-8");
        if content.contains("shitty") {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "bad_words_list": [
                    {
                        "deviations": 0,
                        "end": 16,
                        "info": 2,
                        "original": "shitty",
                        "replacedLen": 6,
                        "start": 10,
                        "word": "shitty"
                    }
                    ],
                    "bad_words_total": 1,
                    "censored_content": "this is a ****** sentence",
                    "content": "this is a shitty sentence"
                })),
                http::StatusCode::OK,
            ))
        } else {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({
                    "bad_words_list": [],
                    "bad_words_total": 0,
                    "censored_content": "",
                    "content": "this is a sentence"
                })),
                http::StatusCode::OK,
            ))
        }
    }

    #[allow(clippy::unused_self)]
    fn build_routes(&self) -> impl Filter<Extract = impl Reply> + Clone {
        warp::post()
            .and(warp::path("bad_words"))
            .and(warp::query())
            .map(|_: HashMap<String, String>| ())
            .and(warp::path::end())
            .and(warp::body::bytes())
            .and_then(Self::check_profanity)
    }

    #[must_use]
    pub fn oneshot(&self) -> OneshotHandler {
        let (tx, rx) = oneshot::channel::<i32>();
        let routes = Self::build_routes(self);

        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(self.socket, async {
            rx.await.ok();
        });

        tokio::task::spawn(server);

        OneshotHandler { sender: tx }
    }
}
