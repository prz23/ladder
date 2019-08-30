FROM ubuntu:18.10
LABEL maintainer="laddernetwork"
LABEL description="This is a docker for ladder node"

WORKDIR /chain

RUN apt-get update \
    && apt-get install -y libssl-dev \
    ca-certificates \
    git \
    wget \
    make \
    g++

RUN wget -qO- https://deb.nodesource.com/setup_10.x | bash

RUN apt-get install -y nodejs

# to fix 'user "undefined" does not have permission to access.'
RUN npm config set user root -g

RUN npm install ladder-cli@0.2.1 -g

RUN rm -rf /var/lib/apt/lists \
    && apt-get autoremove \
    && apt-get clean \
    && apt-get autoclean

COPY ./ladder /chain/
COPY ./entrypoint.sh /chain/

EXPOSE 30333 9933 9944
