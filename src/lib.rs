use std::net;

use tokio::sync::oneshot::{self, Sender};

pub mod config;
pub use config::Config;

pub mod profanity;
use profanity::check_profanity;

pub mod router;
use router::build_routes;

pub mod routes;

pub mod store;
use store::Store;

pub mod types;

pub struct OneShotHandler {
    pub sender: Sender<i32>,
}

pub async fn run(config: Config, store: Store) {
    let routes = build_routes(store).await;
    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;
}

/// # Panics
///
/// Will panic if socket address in not valid.
pub async fn oneshot(store: Store) -> OneShotHandler {
    let routes = build_routes(store).await;
    let (tx, rx) = oneshot::channel::<i32>();

    let socket: net::SocketAddr = "127.0.0.1:3030"
        .to_owned()
        .parse()
        .expect("not a valid address");

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(socket, async {
        rx.await.ok();
    });

    tokio::task::spawn(server);

    OneShotHandler { sender: tx }
}
