# Integration Test Fixtures

This directory contains deterministic fixtures that integration tests clone into a writable
scratch space (see `tests/common.rs::writable_fixture`).

Included assets:

- `topics.json` – baseline public topics used by public-channel tests.
- `users/alice.json`, `users/carol.json` – sample user accounts referenced by forum and
  public command suites.
- `messages/` – kept mostly empty; tests create temporary copies of specific threads as
  needed during execution.

Generated test artifacts (such as additional messages or temporary users) are **not**
checked in. The `.gitignore` in this directory keeps those out of version control while
allowing the canonical fixtures above to remain tracked.
