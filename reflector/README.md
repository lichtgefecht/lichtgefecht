# Reflector

Server component to reflect laser tagger messages

## Building


### Release build

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo doc
cargo build --release
```

## MQTT
For example
```
docker run --rm -p 1883:1883 --name nanomq emqx/nanomq:0.22.10
```