# Games

Meshbbs includes optional, lightweight games you can access from the public channel. They’re designed to be low‑traffic and fun without overwhelming the mesh.

## 🎰 Slot Machine (public channel)

- Commands (use your configured prefix; default shown):
  - `^SLOT` / `^SLOTMACHINE` — spin once; the BBS broadcasts the result on the public channel (best‑effort)
  - `^SLOTSTATS` — show your coins, spins, wins, and jackpots
- Economy:
  - Each spin costs 5 coins
  - New players start with 100 coins
  - If your balance reaches 0, you’ll be refilled to 100 after ~24 hours
- Payouts (multiplier × bet):
  - 7️⃣7️⃣7️⃣ = JACKPOT (progressive pot, minimum 500 coins; grows by 5 coins per losing spin), 🟦🟦🟦 ×50, 🔔🔔🔔 ×20, 🍇🍇🍇 ×14, 🍊🍊🍊 ×10, 🍋🍋🍋 ×8, 🍒🍒🍒 ×5
  - Two 🍒 ×3, one 🍒 ×2, otherwise ×0
- Visibility and reliability:
  - Results are broadcast to the public channel for room visibility (best‑effort)
  - Broadcasts may request an ACK and are considered successful when at least one ACK is received within a short window (no retries)
- Persistence: Player balances and stats are stored under `data/slotmachine/players.json`

Tip: If you see “Out of coins… Next refill in ~Hh Mm”, check back later or run `<prefix>SLOTSTATS` (default `^SLOTSTATS`) to see your current balance and stats.

---

## 🎱 Magic 8‑Ball (public channel)

- Command:
  - `<prefix>8BALL` (default `^8BALL`) — ask a yes/no question and receive a classic Magic 8‑Ball response
- Behavior:
  - Stateless and lightweight; no persistence
  - Broadcast-only on the public channel (best‑effort)
- Reliability:
  - Broadcasts may request an ACK and are considered successful when at least one ACK is received within a short window (no retries)

---

## 🔮 Fortune Cookies (public channel)

- Command:
  - `<prefix>FORTUNE` (default `^FORTUNE`) — receive a random fortune from classic Unix wisdom databases
- Behavior:
  - Stateless; draws from a curated set of fortunes including programming quotes, philosophy, literature, poetry, and humor
  - All fortunes under 200 characters for mesh-friendly transmission
  - Broadcast-only on the public channel (best‑effort)
  - 5-second cooldown per node to prevent spam
- Content:
  - Classic Unix fortune database entries
  - Programming and technology wisdom
  - Motivational quotes and life philosophy
  - Clean humor and wit
- Quality Assurance:
  - Comprehensive unit test coverage (11+ tests)
  - Thread safety validation
  - Content quality checks and character validation
  - Randomness and distribution testing
- Reliability:
  - Same broadcast behavior as Magic 8‑Ball

> 💡 **Developer Note**: The Fortune module includes extensive documentation and testing. See [`docs/development/fortune-module.md`](../development/fortune-module.md) for implementation details.

---

More games may be added over time. Have an idea? Open a GitHub issue or discussion!

---

## 🧭 TinyHack (DM door)

TinyHack is an optional, compact ASCII roguelike playable via DM sessions. It renders a full snapshot each turn and accepts terse commands.

- Enable in `config.toml`:

```toml
[games]
tinyhack_enabled = true
```

- Enter via main menu: press `T` (shown as [T]inyHack when enabled)
- Controls: `N,S,E,W` move; `A` attack; `U P` drink potion; `U B` use bomb; `C F` cast Fireball; `T` take loot; `O` open locked door (needs a key); `R` rest; `I` inspect; `?` help; `B` back to BBS menu
 - Controls: `N,S,E,W` move; `A` attack; `U P` drink potion; `U B` use bomb; `C F` cast Fireball; `T` take loot; `O` open locked door (needs a key); `R` rest; `I` inspect; `?` help; `B` back to BBS menu
 - UI: Each turn shows a compact command legend when space allows, plus LVL and XP progress in the header (e.g., `XP7/12`)
- Goal: Find the Stairs and escape the Tiny Dungeon
- Save files: `data/tinyhack/<username>.json` (atomic write‑then‑rename; fsync)
- Output is ASCII‑only and capped at ~230 bytes per turn