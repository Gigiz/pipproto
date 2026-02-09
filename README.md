# pipproto

PiProto is a lightweight binary framing protocol designed for authenticated, ordered events over pub/sub transports (e.g. MQTT).

This repository contains:
- a reference implementation in Rust
- a mini-RFC specification (draft)

## Status
Work in progress — v1 draft.

## Goals
- Simple binary frame format
- Explicit versioning
- Deterministic encoding/decoding
- Replay resistance (counter-based)
- Transport-agnostic design

## Non-goals
- Encryption (out of scope for v1)
- Reliable delivery
- Global ordering across senders

## Build
```bash
cargo build
```

## Run
```bash
cargo run
```

## Test
```bash
cargo test
```
