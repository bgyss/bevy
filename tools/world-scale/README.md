# World-scale intersection experiment

This fork-local package implements milestone 1 from [`docs/world-scale`](../../docs/world-scale/README.md): a deterministic two-cell intersection, an authoritative 30 Hz headless Bevy server, native and WASM clients, durable checkpoint+journal recovery, and a versioned replay comparator.

The procedural scene uses stable IDs `100` (crate), `200` (pedestrian), `300` (crossing), and `400`/`401` (player spawns). Space pushes the crate; `U` unloads cell 1 and `R` reloads both cells. When the crate reaches the crossing, the pedestrian changes from the direct route to the detour.

All commands below run from the Bevy repository root. If Cargo is not already configured in the shell used by the Codex workspace, prefix commands with:

```sh
export PATH=/workspace/.cargo-home/bin:$PATH
export CARGO_HOME=/workspace/.cargo-home
export RUSTUP_HOME=/workspace/.rustup
export CARGO_TARGET_DIR=/workspace/bevy-target
```

## Server

```sh
cargo run -p bevy_world_scale --bin world_scale_server -- \
  --listen 127.0.0.1:4200 \
  --state /tmp/bevy-world-scale-state
```

The server uses `ScheduleRunnerPlugin` and `FixedUpdate` at 30 Hz. It binds the WebSocket before reporting readiness and durably appends each authoritative tick before sending command acknowledgments.

## Native client

```sh
cargo run -p bevy_world_scale --features client --bin world_scale_client -- \
  --server ws://127.0.0.1:4200 --session 1
```

Start a second copy with `--session 2`. Each receives the same complete manifest and authoritative snapshots.

## Browser client

```sh
rustup target add wasm32-unknown-unknown
cargo build -p bevy_world_scale --features client --bin world_scale_client \
  --target wasm32-unknown-unknown
wasm-bindgen \
  --out-dir tools/world-scale/web \
  --out-name world_scale_client \
  --target web \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/world_scale_client.wasm"
(cd tools/world-scale/web && python3 serve.py)
```

Open `http://127.0.0.1:48000/`. The WASM client connects to `ws://127.0.0.1:4200` and exposes the same Space, `U`, and `R` controls. Generated `.wasm`, `.js`, and TypeScript declaration files are build outputs and are not committed.

## Replay and recovery

```sh
# clean replay
cargo run -p bevy_world_scale --bin world_scale_replay -- \
  --replay tools/world-scale/replays/v1/intersection.json

# known fault; expected nonzero status
cargo run -p bevy_world_scale --bin world_scale_replay -- \
  --replay tools/world-scale/replays/v1/intersection.json \
  --fault accept-duplicate

# full network and restart scenario
bash tools/world-scale/scripts/run_recovery_scenario.sh
```

The scenario starts two native WebSocket observers, duplicates the push, disconnects player A before its acknowledgment, retries after reconnect, unloads/reloads a cell, restarts the server, and late-joins player C. It asserts crate position `[1000, 0]`, crate revision `1`, a single accepted push, and pedestrian route `Detour` after every recovery boundary.
