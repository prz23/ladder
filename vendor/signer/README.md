A simple signer for ethereum and Abos.

# Feature

## interfaces

- sign_message
- sign_transaction


# Hacker

## Dependences

- [protobuf 3.5.1](https://github.com/google/protobuf/releases)
- [rust-protobuf v2.0.4](https://github.com/stepancheg/rust-protobuf)

## Usage

1. Install google protoc

2. Install rust plugin

```
$ cargo install protobuf-codegen --vers 2.0.4
```

3. You can start hack, alter protos than run `./create_protobuf.sh`