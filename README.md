# QUIC test

This is is a minimal code example with the `s2n-quic` Rust crate that causes some problems.

# How to run

Run server:
```bash
cargo r --release -- server 127.0.0.1:10410
```

Run client in a separate terminal:
```bash
cargo r --release -- client 127.0.0.1:10410
```

You should see on stdout that all 16 messages were sent and responses received.