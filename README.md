# Drogue Cloud Event Source

Connects to [Drogue Cloud](https://github.com/drogue-iot/drogue-cloud) integration endpoint, consumes cloud events sent by devices and forwards them to the next service.

Currently it only connects to the Websocket Endpoint.

It can be used in combination with https://github.com/drogue-iot/drogue-postgresql-pusher

## Configuration

| Name | Description | Example |
| ---- | ----------- | ------- |
DROGUE_ENDPOINT | The URL of the endpoint to connect to | wss://ws-integration.sandbox.drogue.cloud |
DROGUE_APP | Drogue application to use | drogue-public-temperature |
DROGUE_USER | Drogue cloud user |
DROGUE_TOKEN | Access token for Drogue cloud | Use `drg admin tokens create` to create one |
K_SINK | The URL of the service to forward events to | http://timescaledb-pusher |
