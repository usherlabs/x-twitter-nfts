FROM rust:1.82 as builder

RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add -
RUN echo 'deb http://dl.google.com/linux/chrome/deb/ stable main' >>   /etc/apt/sources.list
# Install required dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    libssl-dev \
    google-chrome-stable \
    && rm -rf /var/lib/apt/lists/*


WORKDIR /usr/x-twitter-nft/bitte_plugin


# GIT Config necessary for package install
RUN git config --global http.postBuffer 524288000
RUN git config --global core.compression 0

    
# Set environment variable during runtime
ENV CARGO_TARGET_DIR=./src/bitte_plugin
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo install --path=./src/bitte_plugin
CMD ["bitte_plugin"]
