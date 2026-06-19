# Design: Missing Test Scenarios for Stargem Server MVP

## Overview

This document catalogs missing test scenarios for the Stargem server, prioritized by impact and mapped to PRD requirements (`tasks/backlog/prd-stargem-mvp.md`). The goal is to expose latent bugs and coverage gaps in code marked "Complete" by the task tracker (`tasks/backlog/tasks-stargem-mvp.md`).

## Critical Bugs Found During Analysis

Three bugs were identified by reasoning about missing test coverage:

1. **`CombatTickLoop::add_player` spawns dead players** (`src/combat/tick.rs`): `shield_hp`, `armor_hp`, and `energy` are initialized to `0.0` instead of `stats.current_shield`, `stats.current_armor`, `stats.current_energy`. Every player starts with zero HP.

2. **`damage_dealt`/`damage_taken` never written** (`src/game_mode.rs`): `PlayerInMatch` has these fields but no code path updates them. Match history stats always report zero.

3. **`buy_ship` uses nil user ID** (`src/grpc.rs:132`): `Uuid::nil()` is hardcoded instead of reading the authenticated user. Every purchase is attributed to user `00000000-0000-0000-0000-000000000000`.

## Design Approach

### Philosophy

- Tests are written alongside or before bug fixes (TDD where practical)
- Unit tests cover edge cases and error paths; integration tests cover multi-component interactions
- No test requires a running PostgreSQL for basic coverage — database-backed handlers provide fallback paths
- Tests are CI-friendly: pure Rust tests run with `cargo test`, database tests run with `services.postgres` in CI

### Test Levels

| Level | Scope | Dependencies |
|-------|-------|-------------|
| Unit | Single function/module edge cases | None |
| Handler | gRPC handler with mocked state | `MockAuthProvider`, `AppState` |
| Integration | Multi-component (tick loop + physics + damage, gRPC server) | Tokio runtime |
| Database | Schema + seed + SQL queries | PostgreSQL (testcontainers or CI service) |

## Prioritized Test Scenarios

### Phase 1: Bug-Fix Tests (P0 — ship-blocking)

These tests expose the three critical bugs above. Write them first, fix the bugs, then verify.

#### 1.1 `test_add_player_initializes_hp_from_stats`

**File:** `src/combat/tick.rs` (unit test)
**PRD:** FR-6.8, FR-6.1
**Description:** After `add_player` with known `PlayerShipStats`, verify `players["p1"].shield_hp == stats.current_shield`, `armor_hp == stats.current_armor`, `energy == stats.current_energy`.

#### 1.2 `test_damage_stats_accumulate_on_death`

**File:** `src/game_mode.rs` (unit test)
**PRD:** FR-7.5
**Description:** After `on_player_death(p2, Some(p1))`, verify `p1.damage_dealt > 0` and `p2.damage_taken > 0`. This exposes that `damage_dealt`/`damage_taken` are initialized but never updated.

#### 1.3 `test_buy_ship_uses_authenticated_user_id`

**File:** `src/grpc.rs` (handler test)
**PRD:** FR-4.2, US-2
**Description:** Call `buy_ship` via the handler with a known user_id. Verify the SQL query uses that user_id, not `Uuid::nil()`. (Requires adding user_id to the handler signature or extracting from auth context.)

### Phase 2: Weapon Overheat (P0 — FR-6.6)

The overheat mechanic exists as data fields (`heat_per_shot`, `heat_level`) but has zero runtime logic. Add a `WeaponHeatState` or similar and test:

#### 2.1 Heat accumulates on fire
#### 2.2 Overheat threshold blocks further fire
#### 2.3 Cooldown reduces heat over time when not firing
#### 2.4 Overheat forces a longer cooldown period
#### 2.5 Heat clamps to [0, max_heat]

### Phase 3: Active Module Activation (P0 — FR-6.4)

#### 3.1 OneShot deducts energy and starts cooldown
#### 3.2 Ongoing toggle starts/stops energy drain
#### 3.3 Activate rejected when energy < cost
#### 3.4 Activate rejected while on cooldown
#### 3.5 Cooldown decrements each tick
#### 3.6 Max 4 active modules enforced
#### 3.7 Special role module is separate from the 4 active module limit

### Phase 4: Missile Flight Behavior (P0 — FR-6.7)

#### 4.1 Missile moves at configured speed each tick
#### 4.2 Lifetime expiration triggers destruction
#### 4.3 Turn rate limits tracking angle change
#### 4.4 Impact deals damage to target
#### 4.5 Lock-on trajectory tracking
#### 4.6 Target destroyed mid-flight behavior
#### 4.7 Blast radius area damage

### Phase 5: Physics Edge Cases (P1 — FR-6.10)

#### 5.1 dt=0 produces no change in position/velocity/rotation
#### 5.2 Negative dt is clamped to 0
#### 5.3 Very large dt is numerically stable (no NaN/Inf)
#### 5.4 Yaw rotates forward vector correctly
#### 5.5 Pitch rotates forward vector correctly
#### 5.6 Roll rotates around forward axis only
#### 5.7 Multi-axis rotation composes correctly
#### 5.8 Forward vector always unit length after rotation
#### 5.9 Full throttle acceleration reaches speed cap
#### 5.10 Drag reduces velocity but never reverses direction
#### 5.11 Speed cap enforced independently per axis (diagonal thrust)

### Phase 6: Damage Edge Cases (P1 — FR-6.2)

#### 6.1 Negative raw_amount clamped to 0
#### 6.2 EM and Thermic types at shield=0 verify armor multiplier
#### 6.3 `DamageType::from_str` handles ALL case variants
#### 6.4 `load_damage_multipliers` with missing file returns defaults
#### 6.5 `load_damage_multipliers` with malformed TOML returns defaults
#### 6.6 `load_damage_multipliers` with valid file loads custom multipliers

### Phase 7: Game Mode Edge Cases (P1 — FR-7.x)

#### 7.1 Active match blocks new `try_start_match`
#### 7.2 Tie-breaking at time limit with equal scores is well-defined
#### 7.3 Empty queue `try_start_match` returns `None` without panic
#### 7.4 `on_tick` after `is_finished` is a no-op
#### 7.5 Dequeue removes correct player; remaining players shift position
#### 7.6 Match duration tracking accumulates correctly

### Phase 8: Stats Computation Edge Cases (P1 — FR-6.3)

#### 8.1 Zero-modifier module produces identical stats to no modules
#### 8.2 Negative modifier cannot produce negative max stats (clamp)
#### 8.3 Multiple modules stacking on same stat are additive (documented)
#### 8.4 `compute()` always resets current_* = max_* (fresh state)

### Phase 9: gRPC Handler Tests (P0-P1 — FR-4.2)

#### 9.1 **AuthHandler** — login (valid ticket -> session, invalid ticket -> UNAUTHENTICATED), validate_session (valid token -> valid=true, expired token -> valid=false)
#### 9.2 **ShopHandler** — list_ships (with/without DB pool), buy_ship (success, nonexistent model, invalid UUID, insufficient credits, already owned)
#### 9.3 **HangarHandler** — list_hangar (with/without DB), assign_ship_to_slot (success, out-of-range, duplicate, permission denied)
#### 9.4 **LoadoutHandler** — all 4 equip handlers (success, nonexistent module, slot index validation, type constraints, weapon size compatibility)
#### 9.5 **MatchmakingHandler** — queue_for_match (success, already queued, already in match), queue_status (queued, matched, unknown), leave_queue (queued, not queued)
#### 9.6 **MatchHistoryHandler** — get_history (empty, populated, pagination, without DB)
#### 9.7 **AppState** — construction with DB pool `Some(pool)`
#### 9.8 **Error mapping** — all handlers map correctly: UNAUTHENTICATED, INTERNAL, INVALID_ARGUMENT, NOT_FOUND

### Phase 10: Integration Tests (P0-P1)

#### 10.1 Combat tick loop runtime — spawn loop, feed inputs via mpsc, verify snapshots produced at correct rate
#### 10.2 gRPC server integration — start Tonic on random port, call all 6 services with generated clients
#### 10.3 Database schema & seed — verify all 8 tables, 8 ENUMs, FK constraints, 9 seed ship models via `sqlx::test`
#### 10.4 Full damage pipeline — feed shooting input through tick loop, assert damage events in snapshot
#### 10.5 QUIC serialization roundtrip — encode/decode protobuf messages to verify field alignment
#### 10.6 Match lifecycle E2E — queue 4 players, start match, simulate kills, verify scoring, check match history
#### 10.7 Multi-player physics sync — 2 players with different inputs in same tick loop produce independent states

## Implementation Order

```
Phase 1 (bug-fix tests)     -> fix bugs 1-3       -> verify
Phase 2-4 (P0 unit)         -> implement + test    -> verify
Phase 5-8 (P1 unit)         -> implement + test    -> verify
Phase 9 (gRPC handlers)     -> implement + test    -> verify
Phase 10 (integration)      -> implement + test    -> verify
```

## Spec Self-Review

- No placeholders or TODOs remain
- No contradictions between phases
- Scope is focused on test scenarios only (no feature work beyond test-enabling fixes)
- Each requirement is unambiguous — specific test descriptions with expected assertions
