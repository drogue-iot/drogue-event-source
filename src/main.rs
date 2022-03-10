mod config;
mod sender;

use crate::{config::*, sender::Sender};
use anyhow::{bail, Context as AnyhowContext, Result};
use cloudevents::binding::rdkafka::MessageExt;
use futures_util::stream::StreamExt;
use rdkafka::{
    config::FromClientConfig,
    consumer::{Consumer, StreamConsumer},
    util::DefaultRuntime,
};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{
    connect,
    http::{header, Request},
    Message,
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    log::info!("Starting Drogue Event Source!");

    let config = Config::from_env()?;

    if config.endpoint.username.is_some() & config.endpoint.token.is_some() {
        bail!("You must not provide both basic auth and bearer auth");
    }

    let sender = Sender::new(config.k_sink, config.endpoint)?;

    match config.mode {
        Mode::Websocket(config) => {
            log::info!("Using WebSocket mode");
            websocket(config, sender).await
        }
        Mode::Kafka(config) => {
            log::info!("Using Kafka mode");
            kafka(config, sender).await
        }
    }
}

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Failed to receive: {0}")]
    Receive(#[from] rdkafka::error::KafkaError),
    #[error("Failed to parse event: {0}")]
    Event(#[from] cloudevents::message::Error),
}

async fn kafka(config: KafkaConfig, sender: Sender) -> Result<()> {
    let mut kafka_config = rdkafka::ClientConfig::new();

    kafka_config.set("bootstrap.servers", config.bootstrap_servers);
    kafka_config.extend(
        config
            .properties
            .into_iter()
            .map(|(k, v)| (k.replace('_', "."), v)),
    );

    // set up for at-least-once delivery

    kafka_config.set("enable.auto.commit", "true");
    kafka_config.set("auto.commit.interval.ms", "5000");
    kafka_config.set("enable.auto.offset.store", "false");

    // dump config

    log::debug!("Kafka config: {:#?}", kafka_config);

    let consumer = StreamConsumer::<_, DefaultRuntime>::from_config(&kafka_config)?;

    log::info!("Prepare subscribe...");
    consumer.subscribe(&[&config.topic])?;

    log::info!("Starting stream...");
    let mut stream = consumer.stream();

    log::info!("Running stream...");

    loop {
        match stream.next().await.map(|r| {
            r.map_err::<KafkaError, _>(|err| err.into())
                .and_then(|msg| {
                    msg.to_event()
                        .map_err(|err| err.into())
                        .map(|evt| (msg, evt))
                })
        }) {
            None => break,
            Some(Ok(msg)) => match sender.send(msg.1).await {
                Ok(()) => {
                    if let Err(err) = consumer.store_offset_from_message(&msg.0) {
                        log::info!("Failed to ack: {err}");
                        break;
                    }
                }
                Err(err) => {
                    // failed to deliver the event, even with retries
                    log::info!("Failed to deliver event: {}", err);
                    break;
                }
            },
            Some(Err(KafkaError::Receive(err))) => {
                // there is not much we can do here, except to shut down and retry
                log::warn!("Failed to receive from Kafka: {err}");
                break;
            }
            Some(Err(KafkaError::Event(err))) => {
                // again not much we can do, except ignoring the event
                log::warn!("Failed to decode event: {err}");
                continue;
            }
        };
    }

    log::warn!("Exiting stream loop!");

    Ok(())
}

async fn websocket(config: WebsocketConfig, sender: Sender) -> Result<()> {
    let url = format!("{}/{}", config.drogue_endpoint, config.drogue_app);

    let request = Request::builder()
        .uri(url)
        .header(
            header::AUTHORIZATION,
            format!(
                "Basic {}",
                base64::encode(format!("{}:{}", config.drogue_user, config.drogue_token))
            ),
        )
        .body(())?;

    log::info!("Connecting to websocket with request : {:?}", request);
    let (mut socket, response) =
        connect(request).context("Error connecting to the Websocket endpoint:")?;
    log::debug!("HTTP response: {}", response.status());

    loop {
        let msg = socket.read_message();
        let event = match msg {
            Ok(Message::Text(data)) => Some(serde_json::from_str(&data)?),
            Ok(Message::Binary(data)) => Some(serde_json::from_slice(&data)?),
            _ => None,
        };

        if let Some(event) = event {
            log::info!("Sending event: {}", event);
            match sender.send(event).await {
                // FIXME: need verify the status code
                Ok(res) => log::info!("Response: {:?}", res),
                Err(err) => log::warn!("Can't send event: {}", err),
            }
        }
    }
}
