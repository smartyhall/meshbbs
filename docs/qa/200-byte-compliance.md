# 200-Byte Compliance Checklist

This checklist keeps TinyMUSH (and existing games) aligned with the strict
200-byte payload budget enforced by Meshtastic links.  Run through it whenever
you add, refactor, or review text that is delivered to end users.

## 1. Collect candidate payloads

- [ ] Generate the relevant transcript with `cargo test -- --ignored` or the
      specific integration test for your feature.
- [ ] Save any new scripted output under `tests/test-data-int/` using
      descriptive file names (`tinyhack_map_intro.txt`, `tinymush_mayor_greeting.txt`).
- [ ] Update integration or unit tests to assert on the text you captured so
      regressions fail loudly.

## 2. Run automated UTF-8 validation

- [ ] Execute the validator across docs and transcripts:
  ```bash
  just qa-utf8
  ```
- [ ] For ad-hoc files, pass a custom glob or path:
  ```bash
  just utf8-check docs/user-guide/games.md tests/test-data-int/tinymush/*.txt
  ```
- [ ] Resolve any reported lines that exceed the 200-byte limit by tightening
      wording, chunking output, or trimming optional detail.

## 3. Extend test coverage

- [ ] Add assertions guarding byte budgets in the relevant tests (see
      `tests/utf8_chunking_crash.rs` and `tests/tinyhack_minimap.rs` for
      patterns).
- [ ] When budgets are enforced by helper functions (e.g.,
      `BbsServer::chunk_utf8`), ensure new call sites use them.

## 4. Update documentation & checklists

- [ ] Note the scenario in this checklist (add a bullet under
      "Scenario coverage" below) so future contributors know it is guarded.
- [ ] Update `CONTRIBUTING.md` with any new workflows that other contributors
      must follow.
- [ ] If UX copy changed, refresh the relevant sections of
      `docs/user-guide/games.md` and `docs/qa/real-world-test-plan.md`.

## 5. Scenario coverage log

Keep this section current as you harden additional flows.  Link to tests or
transcripts when possible.

- TinyHack minimap fog-of-war (`tests/tinyhack_minimap.rs`) – validated with
  deterministic navigation helper.
- Welcome system greetings (`tests/welcome_system.rs`) – verified chunking via
  `BbsServer::chunk_utf8`.

---

**Tip:** The validator ignores blank lines and comment prefixes (`//`, `#`).
To validate raw output from Meshtastic hardware, pipe it directly:

```bash
meshtastic --no-format --port /dev/ttyUSB0 read | just utf8-check -
```
