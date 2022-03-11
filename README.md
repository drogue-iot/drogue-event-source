# Drogue Cloud Event Source

Connects to [Drogue Cloud](https://github.com/drogue-iot/drogue-cloud) integration endpoint, consumes cloud events sent by devices and forwards them to the next service.

Currently, it only connects to the Websocket Endpoint.

It can be used in combination with https://github.com/drogue-iot/drogue-postgresql-pusher

## Configuration

| Name                      | Description                                                                                                                                                  | Example                          |
|---------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------|
| MODE                      | The source mode                                                                                                                                              | `kafka` of `ws`                  |
| K_SINK                    | The URL of the service to forward events to                                                                                                                  | http://timescaledb-pusher        |
| ENDPOINT__METHOD          | The HTTP method to use, defaults to `POST`                                                                                                                   | `POST`                           |
| ENDPOINT__USERNAME        | Enables basic auth support using the provided username                                                                                                       | `foo`                            |
| ENDPOINT__PASSWORD        | Use as password when basic auth is enabled                                                                                                                   | `bar`                            |
| ENDPOINT__TOKEN           | Enables bearer auth using the provided token                                                                                                                 | `e234c376f48e`                   |
| ENDPOINT__HEADERS__*      | Additional HTTP headers, prefixed with `ENDPOINT__HEADERS__`                                                                                                 | `ENDPOINT_HEADERS_AUTHORIZATION` |
| ENDPOINT__TLS_INSECURE    | Disable TLS validations                                                                                                                                      | `false`                          |
| ENDPOINT__TLS_CERTIFICATE | The certificate of the (only) trust anchor to use for TLS. By default it will use the system's trust anchors. The certificate must in the PEM PKCS#1 format. |                                  |
| ENDPOINT__TIMEOUT         | The timeout of the send operation                                                                                                                            | `15s`                            |
| ENDPOINT__ERROR_DELAY     | The delay before re-trying a failed operation                                                                                                                | `1s`                             |
| ENDPOINT__RETRIES         | The number of re-tries before giving up                                                                                                                      | 5                                |

### Kafka

The following options are available for the `kafka` mode.

| Name              | Description                                                                                                                                                                                                | Example                                                 |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------|
| TOPIC             | The Kafka topic to consume from                                                                                                                                                                            | `events-foo-bar`                                        |
| BOOTSTRAP_SERVERS | The list of Kafka bootstrap servers                                                                                                                                                                        | `kafka-bootstrap:9091`                                  |
| PROPERTIES__*     | Additional properties for the Kafka consumer, prefixed with `PROPERTIES__`. The prefix will be removed from the name, the remaining name will be transformed to lowercase and `_` will be transated to `.` | `PROPERTIES__SASL_USERNAME` will become `sasl.username` |  

You can find the available properties here: https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md

### Websocket

The following options are available for the `ws` mode. 

| Name            | Description                           | Example                                     |
|-----------------|---------------------------------------|---------------------------------------------|
| DROGUE_ENDPOINT | The URL of the endpoint to connect to | `wss://ws-integration.sandbox.drogue.cloud` |
| DROGUE_APP      | Drogue application to use             | `drogue-public-temperature`                 |
| DROGUE_USER     | Drogue cloud user                     |                                             |
| DROGUE_TOKEN    | Access token for Drogue cloud         | Use `drg admin tokens create` to create one |

## Building locally

You can build the image locally using:

```shell
cargo build --release
podman build . -t drogue-event-source
```

As this application links against C based libraries, it may be necessary to replace the container image base
container to match your host system.

This can be done using the `--from` switch. E.g.:

```shell
podman build . -t drogue-event-source --from registry.fedoraproject.org/fedora-minimal:35
```
