mod config;
use tokio_tungstenite::tungstenite::connect;
use tokio_tungstenite::tungstenite::http::{header, Request};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use serde::Deserialize;
use crate::config::ConfigFromEnv;


#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub k_sink: String,
    pub drogue_endpoint: String,
    pub drogue_app: String,
    pub drogue_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    log::info!("Starting Drogue Event Source!");

    let config = Config::from_env()?;

    let url = format!("{}/{}", config.drogue_endpoint, config.drogue_app);

    let request = Request::builder()
        .uri(url)
        .header(header::AUTHORIZATION, config.drogue_token)
        .body(())?;

    log::info!("Connecting to websocket with request : {:?}", request);
    let (mut socket, response) =
        connect(request).context("Error connecting to the Websocket endpoint:")?;
    log::debug!("HTTP response: {}", response.status());

    let client = reqwest::Client::new();
    loop {
        let msg = socket.read_message();
        match msg {
            Ok(m) => {
                // ignore protocol messages, only show text
                if m.is_text() {
                    let body = m.into_text().expect("Invalid message");
                    log::info!("Sending event: {}", body);
                    match client.post(config.k_sink.clone())
                                .header("Content-Type", "application/cloudevents+json")
                                .body(body)
                                .send()
                                .await {
                                    Ok(res) => log::info!("Response: {:?}", res),
                                    Err(err) => log::warn!("Can't send event: {}", err),
                                }
                }
            }
            Err(e) => break Err(anyhow!(e)),
        }
    }
}
