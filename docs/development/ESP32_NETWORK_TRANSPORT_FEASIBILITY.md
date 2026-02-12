# ESP32 LAN Transport Feasibility (Meshtastic over Network)

## Executive summary

Adding support for communicating with an ESP32-based Meshtastic node over the local network (instead of USB serial) is **highly viable** in this codebase, with **moderate implementation effort**.

- **Viability:** High
- **Risk:** Medium (protocol/framing differences and reconnect behavior are the main unknowns)
- **Estimated effort:**
  - **MVP (single network transport):** ~2–4 engineering days
  - **Production-grade transport abstraction + tests + docs:** ~1–2 weeks

## Why this is viable in this repo

### 1) The message protocol path is already centralized and reusable

The reader/writer path already builds and parses Meshtastic protobuf frames in one place (`src/meshtastic/mod.rs`), including:
- framed `ToRadio` writes,
- frame parsing from inbound bytes,
- higher-level event conversion (`TextEvent`),
- ACK/retry control logic.

That means a new transport can mostly focus on replacing the byte stream source/sink, not rewriting BBS logic.

### 2) Server integration is transport-light today

`BbsServer::connect_device()` currently accepts a `port: &str` and delegates to `create_reader_writer_system(...)`. Most of the BBS business logic only consumes channels (`TextEvent`, `OutgoingMessage`, control channels), not raw serial APIs. This is a good seam for introducing transport selection.

### 3) Existing docs/config hint at transport expansion

Config/docs already mention “serial, Bluetooth, TCP/UDP” in comments, but current implementation only opens serial via `serialport`. This indicates the project direction already anticipates additional transports.

## What blocks network transport today

## 1) Hard dependency on `serialport::SerialPort` in runtime types

Current reader/writer internals are typed around `Arc<Mutex<Box<dyn SerialPort>>>`, so non-serial streams (e.g., `tokio::net::TcpStream`) cannot plug in directly without refactoring.

## 2) Constructors assume serial semantics

`MeshtasticDevice::new()` and `create_shared_serial_port()` both unconditionally use `serialport::new(...)`, plus DTR/RTS toggles and serial-specific flushing.

These operations are not meaningful for TCP transport and should become transport-specific behavior.

## 3) CLI and command naming are serial-specific

The CLI help text currently frames device checks as serial checks, and operator UX is “port path”-centric. A network mode needs explicit configuration and diagnostics.

## Recommended implementation approach

## Phase 1: Transport abstraction (minimal change footprint)

Introduce a small transport trait used by reader/writer:

- `trait MeshTransport { fn read(&mut self, ...); fn write_all(&mut self, ...); fn flush(&mut self, ...); ... }`
- Implementations:
  - `SerialTransport` (wraps existing serialport handle)
  - `TcpTransport` (wraps `tokio::net::TcpStream` or std::net stream adapter)

If trait object ergonomics get awkward with async and locking, a pragmatic alternative is an enum:

- `enum TransportHandle { Serial(...), Tcp(...) }`

and switch inside read/write methods.

**Goal:** keep parser/encoder/scheduler/event code untouched.

## Phase 2: Config model update

Extend `[meshtastic]` config to explicit transport selection, e.g.:

```toml
[meshtastic]
transport = "serial" # or "tcp"
port = "/dev/ttyUSB0" # for serial
# host = "192.168.1.50" # for tcp
# tcp_port = 4403
```

Backward compatibility plan:
- If `transport` missing, default to `serial`.
- Keep `port` behavior unchanged for existing users.

## Phase 3: Connection lifecycle + retries

Network links need different resilience policies than USB serial:

- connect timeout,
- reconnect loop with exponential backoff,
- idle detection and keepalive (if needed by Meshtastic TCP endpoint),
- clearer logging around disconnect/reconnect transitions.

## Phase 4: Operator tooling + docs

- Update `check-device` to support transport-aware checks.
- Add examples for ESP32-on-LAN usage.
- Document security caveats (unencrypted LAN API exposure, firewalling).

## Key technical unknowns / validation points

1. **Exact framing parity on the target ESP32 network API**
   - The code currently expects Meshtastic framed protobuf bytes (header + length).
   - Validate that the ESP32 network endpoint uses identical framing to current serial path.

2. **Auth/security requirements on network endpoint**
   - Confirm whether LAN endpoint requires pairing/token/auth or is open on trusted LAN.

3. **Throughput/latency and retry interactions**
   - Current scheduler pacing is optimized for mesh airtime fairness, but network transport may alter timing behavior around ACK handling.

## Suggested test strategy

- Unit tests for transport parsing/writing boundary behavior.
- Integration tests with a mock TCP stream carrying known frame sequences.
- Soak test: forced disconnect/reconnect cycles while sending DMs/broadcasts.
- Real hardware test with ESP32 node on same LAN:
  - startup sync,
  - DM send/ACK,
  - broadcast path,
  - long-running stability (>=24h).

## Bottom line

This feature is a **good candidate for near-term implementation**. The current architecture (reader/writer channels + centralized protocol handling) gives a strong foundation; the main work is introducing a transport abstraction and hardening connection lifecycle for TCP.
