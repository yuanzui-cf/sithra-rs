use sithra_kit::server::server::Client;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub secret: Option<String>,
}
