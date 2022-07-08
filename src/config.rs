use anyhow::Result;
use reqwest::Method;
use serde::Deserialize;
use serde_with::serde_as;
use std::collections::HashMap;
use std::time::Duration;

pub trait ConfigFromEnv<'de>: Sized + Deserialize<'de> {
    fn from_env() -> Result<Self, config::ConfigError> {
        Self::from(config::Environment::default())
    }

    fn from_env_source(source: HashMap<String, String>) -> Result<Self, config::ConfigError> {
        Self::from(config::Environment::default().source(Some(source)))
    }

    fn from_env_prefix<S: AsRef<str>>(prefix: S) -> Result<Self, config::ConfigError> {
        Self::from(config::Environment::with_prefix(prefix.as_ref()))
    }

    fn from(env: config::Environment) -> Result<Self, config::ConfigError>;
}

impl<'de, T: Deserialize<'de> + Sized> ConfigFromEnv<'de> for T {
    fn from(env: config::Environment) -> Result<T, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(env.separator("__"))
            .build()?;
        cfg.try_deserialize()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "mode")]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Kafka(KafkaConfig),
    #[serde(alias = "ws")]
    Websocket(WebsocketConfig),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct KafkaConfig {
    pub topic: String,
    pub bootstrap_servers: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WebsocketConfig {
    pub drogue_endpoint: String,
    pub drogue_app: String,
    pub drogue_user: Option<String>,
    pub drogue_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub endpoint: EndpointConfig,

    pub k_sink: String,

    #[serde(flatten)]
    pub mode: Mode,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct EndpointConfig {
    #[serde(default = "default_method")]
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub method: Method,

    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub token: Option<String>,

    #[serde(default)]
    pub tls_insecure: bool,

    #[serde(default)]
    pub tls_certificate: Option<String>,

    #[serde(default)]
    pub headers: HashMap<String, String>,

    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,

    #[serde(default = "default_error_delay", with = "humantime_serde")]
    pub error_delay: Duration,

    #[serde(default = "default_retries")]
    pub retries: usize,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            method: default_method(),
            username: None,
            password: None,
            token: None,
            tls_insecure: false,
            tls_certificate: None,
            headers: Default::default(),
            timeout: None,
            error_delay: default_error_delay(),
            retries: default_retries(),
        }
    }
}

const fn default_error_delay() -> Duration {
    Duration::from_secs(1)
}

const fn default_retries() -> usize {
    5
}

const fn default_method() -> Method {
    Method::POST
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::*;

    #[test]
    fn test_cfg_kafka() {
        let env = convert_args!(hashmap!(
            "MODE" => "kafka",
            "BOOTSTRAP_SERVERS" => "bootstrap:9091",
            "TOPIC" => "topic",
            "PROPERTIES__FOO_BAR" => "baz",
            "K_SINK" => "http://localhost",
        ));

        let cfg = Config::from_env_source(env).unwrap();

        assert_eq!(
            cfg.mode,
            Mode::Kafka(KafkaConfig {
                topic: "topic".to_string(),
                bootstrap_servers: "bootstrap:9091".into(),
                properties: convert_args!(hashmap!(
                    "foo_bar" => "baz",
                ))
            })
        );
    }

    #[test]
    fn test_cfg_ws() {
        let env = convert_args!(hashmap!(
            "MODE" => "ws",
            "DROGUE_APP" => "app",
            "DROGUE_ENDPOINT" => "endpoint",
            "DROGUE_USER" => "user",
            "DROGUE_TOKEN" => "token",
            "K_SINK" => "http://localhost",
        ));

        let cfg = Config::from_env_source(env).unwrap();

        assert_eq!(
            cfg.mode,
            Mode::Websocket(WebsocketConfig {
                drogue_app: "app".into(),
                drogue_endpoint: "endpoint".into(),
                drogue_user: Some("user".into()),
                drogue_token: Some("token".into()),
            })
        );
    }

    #[test]
    fn test_cfg_endpoint() {
        let env = convert_args!(hashmap!(
            "MODE" => "ws",
            "DROGUE_APP" => "app",
            "DROGUE_ENDPOINT" => "endpoint",
            "DROGUE_USER" => "user",
            "DROGUE_TOKEN" => "token",
            "K_SINK" => "http://localhost",
            "ENDPOINT__METHOD" => "GET",
            "ENDPOINT__HEADERS__foo" => "bar",
        ));

        let cfg = Config::from_env_source(env).unwrap();

        assert_eq!(
            cfg.endpoint,
            EndpointConfig {
                method: Method::GET,
                username: None,
                password: None,
                token: None,
                tls_insecure: false,
                tls_certificate: None,
                headers: convert_args!(hashmap!(
                    "foo" => "bar"
                )),
                timeout: None,
                error_delay: default_error_delay(),
                retries: 5
            }
        );
    }
}
