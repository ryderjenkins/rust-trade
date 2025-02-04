FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Asia/Shanghai

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

RUN curl -L -o substrate-contracts-node.tar.gz https://github.com/paritytech/substrate-contracts-node/releases/download/v0.35.0/substrate-contracts-node-linux.tar.gz && \
    tar -xvzf substrate-contracts-node.tar.gz && \
    mv artifacts/substrate-contracts-node-linux/substrate-contracts-node /usr/local/bin/ && \
    rm -rf artifacts substrate-contracts-node.tar.gz

WORKDIR /substrate

EXPOSE 9944 9933 30333

CMD ["substrate-contracts-node", "--dev", "--rpc-external"]