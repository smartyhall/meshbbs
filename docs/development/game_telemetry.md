# Game Telemetry Groundwork

Phase&nbsp;0 introduces the first instrumentation for tracking how often each game door is used and how long
sessions remain inside those games. This document captures the logging format, the in-process counters that back
future Prometheus/Grafana exports, and the initial dashboards we expect operations to wire up.

## Log events (`target="meshbbs::games"`)

| Event name  | When emitted                                    | Key fields                                                                                                                                                   |
|-------------|--------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `game.entry` | After the user passes through a `GameDoor`      | `slug`, `session`, `user`, `node`, `command`, `entries`, `active`, `peak`                                                                                    |
| `game.exit`  | Whenever the user leaves an active game session | `slug`, `session`, `user`, `node`, `reason`, `command` (when applicable), `entries`, `exits`, `active`, `peak`                                               |

- `slug`: canonical identifier for the game door (`tinyhack`, `tinymush`, etc.).
- `session`: server session identifier (stable for duration of the connection).
- `user`: sanitized display name (falls back to `Guest` before login).
- `node`: radio/node identifier associated with the session.
- `command`: the menu command that initiated the transition (upper-cased and sanitized).
- `reason`: `command`, `logout`, or future programmatic reasons (timeouts, disconnects).
- `entries` / `exits`: monotonically increasing counters per slug.
- `active`: current number of concurrent sessions inside the game.
- `peak`: historical max of `active` since process start.

All log fields are sanitized via `logutil::escape_log`, keeping newlines and control characters out of the output so
collectors (Fluent Bit, Vector, etc.) do not misparse payloads.

## Runtime counters (`meshbbs::metrics`)

The `metrics` module now tracks per-game counters in memory:

- `record_game_entry(slug)` increments entry totals, concurrent player counts, and peak counters.
- `record_game_exit(slug)` decrements concurrency and increments exit totals (saturating at zero when needed).
- `game_counters_snapshot()` returns a clone of the current counter map for exporting through an HTTP handler or a
  metrics task in later phases.

A lightweight unit test (`metrics::tests::game_entry_exit_updates_counters`) guards these semantics.

### Suggested export shape

When we expose metrics externally (Phase&nbsp;3 telemetry milestone), the snapshot can be rendered into gauges and
counters:

- `meshbbs_game_entries_total{slug="tinyhack"}`
- `meshbbs_game_exits_total{slug="tinyhack"}`
- `meshbbs_game_active_sessions{slug="tinyhack"}`
- `meshbbs_game_active_peak{slug="tinyhack"}`

## Dashboard starter kit

Operations should prepare the following Grafana panels using the structured logs (until a metrics endpoint ships):

1. **Concurrent players by game** – stacked area chart over time using `game.entry` / `game.exit` events aggregated by
   `slug`, applying `.active` as the primary series.
2. **Entry vs. exit volume** – bar chart grouped hourly to catch spikes and identify players getting stuck inside
   doors.
3. **Session duration distribution** – histogram derived from pairing `game.entry` and `game.exit` log timestamps per
   `session`/`slug`.
4. **Ad-hoc alerts** – count `game.exit` records with `reason=logout` vs future disconnect reasons to spot crashes or
   dropped links.

Document any derived dashboards in the ops runbook once wiring is complete. This page will grow as we formalize the
export format in later phases.
