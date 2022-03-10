use crate::EndpointConfig;
use anyhow::bail;
use cloudevents::binding::reqwest::RequestBuilderExt;
use reqwest::Url;
use std::time::Duration;
use thiserror::Error;

pub struct Sender {
    config: EndpointConfig,
    url: Url,
    client: reqwest::Client,
}

#[derive(Clone, Debug, Error)]
pub enum SendError {
    #[error("Temporary publish error: {0}")]
    Temporary(String),
    #[error("Permanent publish error: {0}")]
    Permanent(String),
}

impl Sender {
    pub fn new(url: String, config: EndpointConfig) -> anyhow::Result<Self> {
        let mut client = reqwest::Client::builder();

        if config.tls_insecure {
            client = client
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);
        }

        client = client.timeout(
            config
                .timeout
                .clone()
                .unwrap_or_else(|| Duration::from_secs(5)),
        );

        let client = client.build()?;

        let url = Url::parse(&url)?;

        Ok(Sender {
            config,
            url,
            client,
        })
    }

    pub async fn send_once(&self, event: cloudevents::Event) -> Result<(), SendError> {
        let mut request = self
            .client
            .request(self.config.method.clone(), self.url.clone());

        if let Some(username) = &self.config.username {
            request = request.basic_auth(username, self.config.password.as_ref());
        }

        match request
            .event(event)
            .map_err(|err| {
                SendError::Permanent(format!("Failed to encode cloud event to request: {err}"))
            })?
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) if resp.status().is_server_error() => Err(SendError::Temporary(format!(
                "Server error ({}): {:?}",
                resp.status(),
                resp.text().await.unwrap_or_else(|_| "<unknown>".into())
            ))),
            Ok(resp) if resp.status().is_client_error() => Err(SendError::Permanent(format!(
                "Client error ({}): {:?}",
                resp.status(),
                resp.text().await.unwrap_or_else(|_| "<unknown>".into())
            ))),
            Ok(resp) => Err(SendError::Permanent(format!(
                "Unknown error ({}): {:?}",
                resp.status(),
                resp.text().await.unwrap_or_else(|_| "<unknown>".into())
            ))),
            Err(err) => Err(SendError::Temporary(format!(
                "Failed to perform request: {err}"
            ))),
        }
    }

    pub async fn send(&self, event: cloudevents::Event) -> anyhow::Result<()> {
        let mut attempts = 1;
        loop {
            match self.send_once(event.clone()).await {
                Ok(_) => break Ok(()),
                Err(SendError::Temporary(reason)) => {
                    log::info!(
                        "Received temporary error, retrying: {reason}, attempt: {0}",
                        attempts
                    );
                }
                Err(SendError::Permanent(reason)) => {
                    log::info!("Received permanent error, skipping event: {reason}");
                    return Ok(());
                }
            }

            if attempts > self.config.retries {
                bail!("Giving up to send after {0} attempts", attempts);
            }

            attempts += 1;

            // delay
            tokio::time::sleep(self.config.error_delay).await;
        }
    }
}
