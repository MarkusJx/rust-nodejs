FROM ubuntu:22.04

RUN apt-get update && apt-get upgrade -y &&\
    apt-get install -y python3 build-essential libclang-dev curl ccache git

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash - &&\
    apt-get install -y nodejs

ENV PATH="$PATH:/root/.cargo/bin"
RUN cargo install bindgen-cli

ENV LIBNODE_NODE_VERSION="v21.7.3"
ENV CCACHE_DIR="/app/.ccache"
ENV CC="ccache gcc"
ENV CXX="ccache g++"
WORKDIR /app
