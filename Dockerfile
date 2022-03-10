FROM registry.access.redhat.com/ubi8/ubi:latest as builder

RUN dnf -y install openssl openssl-devel gcc gcc-c++ make cyrus-sasl-devel cmake perl xz

ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH "$PATH:$CARGO_HOME/bin"

RUN mkdir -p /usr/src/drogue-event-source
ADD . /usr/src/drogue-event-source

WORKDIR /usr/src/drogue-event-source

RUN cargo build --release

FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

LABEL org.opencontainers.image.source="https://github.com/drogue-iot/drogue-event-source"

RUN microdnf install -y cyrus-sasl-lib

COPY --from=builder /usr/src/drogue-event-source/target/release/drogue-event-source /

ENTRYPOINT [ "/drogue-event-source" ]
