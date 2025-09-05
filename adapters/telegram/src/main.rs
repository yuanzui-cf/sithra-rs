use axum::Router;
use serde::{Deserialize, Serialize};
use sithra_kit::{plugin, types::initialize::Initialize};
use tokio::net::TcpListener;

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
}

fn default_telegram_api() -> String {
    "https://api.telegram.org".to_owned()
}

fn default_host() -> String {
    "127.0.0.1".to_owned()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (
        plugin,
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
    } = config;

    let app: Router = Router::new();

    let listener = TcpListener::bind((host.as_str(), port)).await?;

    let serve = axum::serve(listener, app);

    tokio::select! {
        _ = plugin.run().join_all() => {},
        _ = tokio::signal::ctrl_c() => {},
        _ = serve => {},
    }

    Ok(())
}
