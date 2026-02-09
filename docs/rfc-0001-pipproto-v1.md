# RFC 0001 — PiProto v1.0

**Title:** Authenticated Event Framing Protocol  
**Status:** Draft  
**Version:** 1.0  
**Last Updated:** 2026-02-09

---

## 1. Abstract
PiProto defines a lightweight, deterministic binary framing format for
authenticated and ordered events over pub/sub transports.

---

## 2. Goals
- Deterministic binary encoding
- Explicit versioning
- Replay resistance via monotonic counters
- Transport-agnostic design

---

## 3. Non-Goals
- Confidentiality (encryption)
- Reliable delivery
- Global ordering across senders

---

## 4. Transport Assumptions
The transport:
- preserves message boundaries
- may reorder or duplicate messages
- must not modify payload bytes

---

## 5. Frame Format (v1)

All multi-byte fields are big-endian.

### Header (fixed length: 21 bytes)

| Field      | Size | Description |
|------------|------|-------------|
| Magic      | 2    | ASCII "PP" |
| Version    | 1    | 0x01 |
| MsgType    | 1    | Message type |
| Flags      | 1    | bit0=ACK_REQUIRED, bits1..7 reserved=0 |
| DeviceID   | 8    | Opaque sender identifier |
| Counter    | 8    | Monotonic per DeviceID |

### Body
- Variable-length bytes
- Application-defined semantics

---

## 6. Message Types

| Value | Name    |
|-------|---------|
| 0x01  | EVENT   |
| 0x02  | COMMAND |
| 0x03  | ACK     |
| 0x04  | ERROR   |

Unknown message types MUST be rejected.

---

## 7. Flags
- bit 0: ACK_REQUIRED
- bits 1–7: reserved (MUST be zero)

Frames with reserved bits set MUST be rejected.

---

## 8. Replay Protection
Each sender maintains a monotonically increasing Counter.
Receivers MUST reject frames with counters less than or equal to the last
accepted value for the same DeviceID.

---

## 9. Security (Planned)
Future versions MAY include an authentication tag (e.g. HMAC-SHA256)
computed over header and body bytes.

---

## 10. Compliance
An implementation is compliant with v1 if it:
- validates magic and version
- validates message type and flags
- enforces replay protection
