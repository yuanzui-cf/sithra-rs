use sithra_kit::server::server::Client;

use crate::types::User;

#[derive(Clone)]
pub struct AppState {
    // Sithra Client
    pub client: Client,

    pub secret:   Option<String>,
    pub req:      reqwest::Client,
    pub base_api: reqwest::Url,

    pub bot: User,
}
