# Drogue ClouD Event Source

Connects to Drogue Cloud integration endpoint, consumes cloud events sent by devices and forwards them to the next service.

Currently it only connects to WebSocket Endpoint.

It can be used in combination with https://github.com/drogue-iot/drogue-postgresql-pusher

## Configuration

| Name | Description | Example |
| ---- | ----------- | ------- |
DROGUE_ENDPOINT | The URL of the endpoint to connect to | wss://ws-integration.sandbox.drogue.cloud |
DROGUE_APP | Drogue application to use | drogue-public-temperature |
DROGUE_TOKEN | Token for authenticating with the cloud |
K_SINK | The URL of the service to forward events to | http://timescaledb-pusher |
