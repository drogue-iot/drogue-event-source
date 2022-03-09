FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

LABEL org.opencontainers.image.source="https://github.com/drogue-iot/drogue-event-source"

COPY target/release/drogue-event-source /

ENTRYPOINT [ "/drogue-event-source" ]
