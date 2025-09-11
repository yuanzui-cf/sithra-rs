mod state;
mod types;
mod webhook;

use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};
use sithra_kit::{
    plugin,
    types::initialize::{Initialize, PluginInitError},
};
use tokio::net::TcpListener;

use crate::{
    state::AppState,
    types::{Response, User},
};

#[derive(Clone, Serialize, Deserialize)]
struct Config {
    /// Telegram Bot API URL, default to `https://api.telegram.org`
    #[serde(rename = "telegram-api", default = "default_telegram_api")]
    telegram_api: String,
    /// Telegram Bot Token. Get it from `BotFather`.
    token:        String,

    /// Webhook server port
    port:   u16,
    /// Webhook server host, default to `127.0.0.1`
    #[serde(default = "default_host")]
    host:   String,
    /// Domain for webhook, if not set, will use `{host}:{port}`
    domain: Option<String>,
    /// Secret token for webhook
    ///
    /// A secret token to be sent in a header “X-Telegram-Bot-Api-Secret-Token”
    /// in every webhook request, 1-256 characters. Only characters A-Z, a-z,
    /// 0-9, _ and - are allowed. The header is useful to ensure that the
    /// request comes from a webhook set by you.
    secret: Option<String>,
}

fn default_telegram_api() -> String {
    "https://api.telegram.org".to_owned()
}

fn default_host() -> String {
    "127.0.0.1".to_owned()
}

#[tokio::main]
async fn main() {
    let (
        mut plugin,
        Initialize {
            config,
            id: plugin_id,
            ..
        },
    ) = plugin!(Config);

    let Config {
        telegram_api,
        token,
        port,
        host,
        domain,
        secret,
    } = config;

    let req = reqwest::Client::new();
    let base_api = plugin.expect(reqwest::Url::parse(&telegram_api)).await;
    let base_api = plugin.expect(base_api.join(&format!("/bot{token}/"))).await;

    let get_me = plugin.expect(base_api.join("./getMe")).await;

    let res = plugin
        .expect(req.post(get_me.clone()).send().await)
        .await
        .json::<Response<User>>()
        .await;
    let res = plugin.expect(res).await;

    if !res.ok {
        plugin
            .err(PluginInitError::CustomError(
                "Failed to get bot info. Check if the token is correct.".to_owned(),
            ))
            .await;
    }

    let state = AppState {
        client: plugin.server.client(),
        secret,
        req,
        base_api,
        bot: res.result.unwrap(),
    };

    let app: Router = Router::new().route("/webhook", post(webhook::webhook)).with_state(state);

    let listener = plugin.expect(TcpListener::bind((host.as_str(), port)).await).await;

    let serve = axum::serve(listener, app);

    log::info!("Server started");

    tokio::select! {
        _ = plugin.run().join_all() => {},
        _ = tokio::signal::ctrl_c() => {},
        _ = serve => {},
    }
}
