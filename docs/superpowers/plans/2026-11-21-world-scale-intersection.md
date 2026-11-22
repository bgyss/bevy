# World-Scale Intersection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the first runnable, deterministic world-scale intersection experiment with a headless authoritative server, native and WASM observers, durable recovery, and a replay comparator that proves required failure cases.

**Architecture:** Add a fork-local `tools/world-scale` workspace package. A display-independent Bevy plugin owns stable-ID simulation state and a manual 30 Hz tick; binaries reuse it for replay, a headless server using `ScheduleRunnerPlugin`, and native/WASM clients. The server persists an atomic checkpoint plus tick journal, deduplicates `(session_id, sequence)` commands, and publishes versioned JSON snapshots over WebSocket.

**Tech Stack:** Rust, Bevy 0.20-dev path dependency, serde/serde_json, native and browser WebSocket adapters, procedural Bevy meshes/sprites, filesystem checkpoint+journal on native targets.

**Spec:** `docs/world-scale/README.md`, `docs/world-scale/simulation.md`, and `docs/world-scale/benchmarks.md`

## Global Constraints

- Keep all experiment code in `tools/world-scale`; do not modify engine crates.
- Run authority at 30 Hz using Bevy fixed schedules and the headless schedule runner.
- Use stable object IDs and versioned fixture, protocol, checkpoint, journal, and replay schemas.
- Commit authoritative state and deduplication outcome together before acknowledging a command.
- Use seed 42, two adjacent cells, two player spawns, one crate, one crossing, and one pedestrian.
- Comparator output must identify the first differing tick, stable object ID, and field.
- Native and WASM clients must consume the same manifest and server protocol.

## Review Focus

- A duplicate `(session_id, sequence)` arriving after restart returns the original result without moving the crate again; pin in Task 2 recovery tests.
- A truncated final journal record is ignored while earlier committed records recover; pin in Task 2 journal tests.
- A client that unloads and reloads a cell rebuilds stable-ID mappings from a complete interest snapshot; pin in Task 3 protocol tests.
- A late join receives the current committed crate and pedestrian state rather than fixture defaults; pin in Task 3 server tests.
- A replay schema or manifest version mismatch fails with a clear diagnostic rather than silently comparing incompatible runs; pin in Task 1 comparator tests.

---

### Task 1: Deterministic fixture, simulation, replay, and comparator

**Files:**

- Create: `tools/world-scale/Cargo.toml`
- Create: `tools/world-scale/src/lib.rs`
- Create: `tools/world-scale/src/model.rs`
- Create: `tools/world-scale/src/fixture.rs`
- Create: `tools/world-scale/src/simulation.rs`
- Create: `tools/world-scale/src/replay.rs`
- Create: `tools/world-scale/src/bin/world_scale_replay.rs`
- Create: `tools/world-scale/replays/v1/intersection.json`

**Interfaces:**

- Produces: `WorldScalePlugin`, `Fixture::intersection(seed: u64)`, `Simulation::tick(&[Command]) -> TickRecord`, `run_replay`, and `compare_runs`.

- [ ] Write failing unit tests for stable fixture IDs, one accepted push, duplicate delivery, detour activation, settling, comparator field diagnostics, version mismatch, and the deliberate duplicate-acceptance fault.
- [ ] Run `cargo test -p bevy_world_scale --lib` and verify the new tests fail.
- [ ] Implement versioned models, fixture generation, fixed-tick systems, replay loading, JSONL recording, and first-difference comparison.
- [ ] Run the clean replay and verify exact agreement; run `--fault accept-duplicate` and verify a nonzero exit naming the first tick, crate ID, and changed field.
- [ ] Commit the deterministic replay slice.

### Task 2: Durable checkpoint, journal, and crash recovery

**Files:**

- Create: `tools/world-scale/src/persistence.rs`
- Modify: `tools/world-scale/src/simulation.rs`
- Modify: `tools/world-scale/src/replay.rs`

**Interfaces:**

- Consumes: versioned `WorldState`, `AcceptedCommand`, and `TickRecord` from Task 1.
- Produces: `DurableStore::open`, `commit_tick`, `checkpoint`, and `recover`.

- [ ] Write failing tests for checkpoint+journal recovery, lost acknowledgment followed by retry, duplicate retry after restart, partial-tail rejection, and autonomous pedestrian progress recovery.
- [ ] Run the focused persistence tests and verify failure.
- [ ] Implement atomic checkpoint replacement and append-only length/checksum-framed tick journal with fsync before acknowledgment.
- [ ] Run persistence and replay tests and verify committed crate/pedestrian state and dedup outcomes recover.
- [ ] Commit the recovery slice.

### Task 3: Authoritative server and client protocol

**Files:**

- Create: `tools/world-scale/src/protocol.rs`
- Create: `tools/world-scale/src/server.rs`
- Create: `tools/world-scale/src/bin/world_scale_server.rs`
- Create: `tools/world-scale/src/client_state.rs`

**Interfaces:**

- Consumes: simulation and durable store from Tasks 1-2.
- Produces: versioned `ClientMessage`/`ServerMessage`, WebSocket server, subscriptions, complete interest snapshots, and reconnect tokens.

- [ ] Write failing protocol/server tests for two overlapping observers, duplicate command delivery, dropped ack retry, reconnect, unload/reload, restart, late join, bounded superseding snapshots, malformed input, and protocol-version mismatch.
- [ ] Run focused server tests and verify failure.
- [ ] Implement a headless `ScheduleRunnerPlugin` server with 30 Hz fixed authority, reliable command handling, cell subscriptions, acknowledgments, deltas, and periodic complete snapshots.
- [ ] Run server tests and a scripted two-client loopback session; verify the push applies once and both observers receive the detour.
- [ ] Commit the networking slice.

### Task 4: Procedural native and WASM clients

**Files:**

- Create: `tools/world-scale/src/presentation.rs`
- Create: `tools/world-scale/src/client_transport.rs`
- Create: `tools/world-scale/src/bin/world_scale_client.rs`
- Create: `tools/world-scale/web/index.html`
- Create: `tools/world-scale/web/serve.py`

**Interfaces:**

- Consumes: protocol messages and client stable-ID view state from Task 3.
- Produces: native/WASM client entry point, procedural two-cell intersection rendering, keyboard push/subscription controls, and structured client event traces.

- [ ] Write failing client-state tests for spawn/update/despawn mapping, interpolation samples, cell unload/reload, and reconnect from an empty cache.
- [ ] Run focused client tests and verify failure.
- [ ] Implement procedural placeholder geometry and target-specific WebSocket adapters while sharing presentation and protocol code.
- [ ] Build and run the native client against the server; build `wasm32-unknown-unknown` and serve the browser bundle.
- [ ] Exercise the browser client if the environment supports WebGPU/WebGL and record precisely which native/browser paths ran.
- [ ] Commit the client slice.

### Task 5: End-to-end scenario, commands, and evidence

**Files:**

- Create: `tools/world-scale/scripts/run_recovery_scenario.sh`
- Create: `tools/world-scale/README.md`
- Create: `tools/world-scale/evidence/run-record.md`
- Modify: `docs/world-scale/README.md`

**Interfaces:**

- Consumes: all binaries and schemas from Tasks 1-4.
- Produces: exact operator commands, reproducible scenario automation, traces, and an honest execution record.

- [ ] Write the scripted scenario assertions covering duplicate delivery, lost acknowledgment, reconnect, unload/reload, server restart, late join, exactly-once push, and committed crate/pedestrian recovery.
- [ ] Run the scenario from a clean temporary state directory and verify every assertion passes.
- [ ] Run the deliberate fault and verify the comparator fails with tick, stable ID, and field.
- [ ] Run formatting, package tests, clippy where practical, native server/client smoke tests, WASM build, and browser smoke when available; record exact outputs and limitations.
- [ ] Document exact server, native client, browser client, replay, comparator, and recovery commands and commit the final evidence slice.
