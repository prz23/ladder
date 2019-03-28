FROM ubuntu:18.04
LABEL maintainer="abmatrix"
LABEL description="This is a abmatrix runtime"

WORKDIR /chain

RUN apt-get update \
    && apt-get install -y libssl-dev \
                          ca-certificates

EXPOSE 30333 9933 9944
