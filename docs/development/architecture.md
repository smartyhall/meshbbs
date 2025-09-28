# Architecture Overview

This document provides a high-level overview of MeshBBS components and their interactions, focusing on the async event loop and periodic timers.

## System Diagram

```mermaid
flowchart TD
    M[Meshtastic Device]
  SIO[Serial (USB/UART)]
    R[Meshtastic Reader Task]
    W[Meshtastic Writer Task]
    SV[BBS Server]
    SCH[Scheduler]
    SESS[Sessions]
    PST[Public State]
    STOR[Storage Layer]
    MSGDB[Message DB]
    USERDB[User DB]
    CFG[Configuration]
    WX[(Weather Service)]

    M <--> SIO
    SIO --> R
    W --> SIO

    %% Event ingress from radio
    R -- TextEvent (mpsc) --> SV
    R -- our_node_id (mpsc) --> SV

    %% Outgoing path via scheduler/ writer
    SV -- Outgoing (mpsc) --> SCH
    SCH -- dispatch --> W

    %% Server subsystems
    SV --> SESS
    SESS -->|per-node| SV
    SV --> PST
    SV --> STOR
    STOR --> MSGDB
    STOR --> USERDB
    SV --> CFG
    SV --> WX
```

## Components

- Meshtastic Reader Task
  - Reads framed bytes from the serial/Bluetooth interface, parses protobuf messages.
  - Emits high-level TextEvent items to the server via mpsc.
  - Extracts our node ID from MyInfo and sends it to the server via a dedicated mpsc channel.
  - Updates the node cache as node summaries and configs arrive.

- Scheduler
  - Receives outgoing message envelopes from the server via mpsc.
  - Enforces pacing (min_send_gap_ms), applies post-DM broadcast gap, and schedules retries/backoffs for DMs.
  - Dispatches due messages to the Writer.

- Meshtastic Writer Task
  - Encodes messages to protobuf frames and writes to the serial/Bluetooth interface.
  - Cooperates with the scheduler’s cadence; reports basic stats/logs.

- BBS Server (Core)
  - Owns sessions, public state, storage handles, and configuration.
  - Drives periodic housekeeping (weather polling, node cache cleanup, ident beacon checks).
  - Routes incoming TextEvents to public/DM handlers and enqueues outbound messages.

- Storage Layer
  - JSON-backed storage for users, topics/subtopics, messages, replies, and audit trails.
  - Safe file locking ensures consistency under concurrent access.

## Event Loop & Timers

The server runs a tokio::select!-based event loop fed by:

1) Text events
   - Source: Meshtastic Reader via mpsc.
   - Effect: Route to public or direct message handlers, mutate session state, reply via scheduler.

2) Internal messages
   - Source: server subsystems and admin flows (unbounded channel).
   - Effect: Lightweight coordination, logging, and admin actions.

3) Periodic tick (1s)
   - Housekeeping driver that runs regardless of radio traffic.
   - Weather: polls every 5 minutes when enabled.
   - Node cache cleanup: runs approximately every hour.
   - Ident Beacon: checked every tick with UTC boundary alignment and de-duplication.

### Ident Beacon Scheduling

- Configuration: `[ident_beacon]` with `enabled` and `frequency` ("5min", "15min", "30min", "1hour", "2hours", "4hours").
- UTC boundary alignment: sends on exact boundaries (e.g., :00, :15, :30, :45 for 15min).
- Startup gating:
  - If a physical device is present, waits for `initial_sync_complete()`.
  - In reader/writer mode, waits to learn `our_node_id` from the reader; falls back after ~120s if still unknown.
- Duplicate prevention:
  - Tracks the epoch-minute of the last send; at most one ident per scheduled minute boundary.

### Scheduler Cadence & Reliability

- Enforces minimum send gap across messages to reduce airtime contention.
- Supports DM retry with bounded backoffs (configurable); broadcasts remain best-effort.
- Optional broadcast ACK confirmation: can request an ACK and treat any single ACK as basic delivery.

## Error Handling Philosophy

- Reader resiliency: transient serial errors are logged and the task continues; exits only on controlled shutdown.
- Server isolation: per-session failures don’t affect others; storage errors are logged and retried when reasonable.
- Clear logging: INFO for key lifecycle events, DEBUG for detailed diagnostics.

## Related Documentation

- Library API docs (rustdoc) provide module-level details and examples.
- See `README.md` for quick start, features, and user-facing documentation.
