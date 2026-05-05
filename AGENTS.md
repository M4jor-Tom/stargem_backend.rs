# Stargem Server — Agent Notes

## Project
- Rust 2021 game server for a space MMORPG (Stargem)
- Single crate: `stargem-server` (bin + lib)
- Custom TCP protocol with length-prefixed JSON messages (4-byte big-endian header + JSON body)
- Optional TLS via `rustls`

## Commands
- `cargo build` — build
- `cargo run` — start server (needs `DATABASE_URL` env)
- `cargo test` — unit + integration tests
- `cargo test --test '*integration*'` — integration tests only (need Postgres)
- `cargo clippy --all-features -- -D warnings` — lint (CI fails on warnings)
- `cargo fmt --all -- --check` — format check
- `cargo audit` — security audit

## Dev Setup
- Postgres 16 required. Start dev DB: `podman-compose -f compose/docker-compose.dev.yml up -d` (port 5432)
- Default `DATABASE_URL`: `postgres://stargem:stargem@localhost/stargem`
- Schema is applied via `compose/init.sql` mounted into the dev container at startup
- Nix dev shell available: `nix develop` (includes rustc, cargo, clippy, rustfmt, sqlx-cli, podman)

## Testing
- Tests use `TEST_DATABASE_URL` env var. CI uses `postgres://stargem_test:stargem_test@localhost:5433/stargem_test`
- Start test DB: `podman-compose -f compose/docker-compose.test.yml up -d` (port 5433)
- Integration tests require Postgres running + schema initialized via `compose/init.sql`
- `serial_test` crate used — tests that touch DB may need serialization
- Helper test UUIDs in `tests/common.rs`: `00000000-0000-0000-0000-000000000001` (ship model), `00000000-0000-0000-0000-000000000002` (weapon)
- Nix integration test shortcut: `nix run .#integration-test`

## CI Order
1. `cargo test --all-features` (with Postgres service on 5433)
2. `cargo clippy --all-features -- -D warnings`
3. `cargo fmt --all -- --check`
4. Integration tests (separate job, needs `init.sql` applied)
5. `cargo audit` (security)

## Architecture
```
src/
  main.rs          — entrypoint, wires DB pool → repos → GameService → GameServer
  lib.rs           — pub modules: api, db, domain, error, game, network, security
  api/             — gRPC-like GameService (tonic + prost), handles ClientMessage → ServerMessage
  db/              — sqlx + Postgres repos (users, ships, ship_models, hangars), schema.sql
  domain/          — entities: User, Ship, ShipStats, GameInstance, DamageType, etc.
  game/            — CombatSystem, GameInstanceManager, Matchmaker
  network/         — GameServer (raw TCP), SessionManager, TLS, protocol (ClientMessage/ServerMessage)
  security/        — rate limiting
  error.rs         — AppError enum (thiserror)
```

## Key Env Vars
| Var | Default | Purpose |
|-----|---------|---------|
| `DATABASE_URL` | `postgres://stargem:stargem@localhost/stargem` | Main DB connection |
| `TEST_DATABASE_URL` | (none) | Test DB connection |
| `BIND_ADDR` | `0.0.0.0:8080` | Server listen address |
| `USE_TLS` | `false` | Enable TLS |
| `TLS_CERT` / `TLS_KEY` | (required if TLS) | PEM cert/key paths |
| `RUST_LOG` | `info` | Tracing filter; dev shell sets `stargem_server=debug,info` |

## Gotchas
- No `.env` file support — env vars must be set explicitly
- No `build.rs` — prost types appear to be hand-written, not codegen'd from `.proto`
- Network layer uses raw TCP with custom framing, NOT gRPC despite tonic/prost deps
- `dashmap` and `parking_lot` used for in-memory concurrency (game state, sessions)
- `sqlx` compile-time query checking requires `DATABASE_URL` at build time or offline mode
