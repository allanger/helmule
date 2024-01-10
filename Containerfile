FROM rust:1.70.0-alpine3.18 as builder
WORKDIR /src
RUN apk update && apk add --no-cache gcc musl-dev
COPY ./ .
RUN rustup default nightly && rustup update
RUN cargo build --release --jobs 2 -Z sparse-registry 

FROM ghcr.io/allanger/dumb-downloader as dudo
RUN apt-get update -y && apt-get install tar git -y
ARG HELM_VERSION=v3.13.3
ARG YQ_VERSION=v4.40.5
ENV RUST_LOG=info
RUN dudo -l "https://get.helm.sh/helm-{{ version }}-{{ os }}-{{ arch }}.tar.gz" -d /tmp/helm.tar.gz -p $HELM_VERSION
RUN dudo -l "https://github.com/mikefarah/yq/releases/download/{{ version }}/yq_{{ os }}_{{ arch }}.tar.gz" -d /tmp/yq.tar.gz -p $YQ_VERSION
RUN tar -xf /tmp/helm.tar.gz  -C /tmp && rm -f /tmp/helm.tar.gz 
RUN tar -xf /tmp/yq.tar.gz  -C /tmp && rm -f /tmp/yq.tar.gz 
RUN mkdir /out
RUN cp `find /tmp | grep helm` /out/
RUN mv `find /tmp | grep yq_` /out/yq
RUN chmod +x /out/helm
RUN chmod +x /out/yq

FROM alpine:3.18
RUN apk update && apk add --no-cache git
COPY --from=builder /src/target/release/helmule /bin/helmule
COPY --from=dudo /out/ /usr/bin
WORKDIR /workdir
ENTRYPOINT ["/bin/helmule"]


