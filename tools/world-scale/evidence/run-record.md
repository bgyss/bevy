# World-scale intersection run record

Date: 2026-11-21 (America/Los_Angeles)

## Automated results

- `cargo test -p bevy_world_scale --features client`: 11 passed, 0 failed.
- Native client check and build: passed after selecting X11-only client platform features; the broader Bevy `2d` feature could not build because this image lacks `wayland-client.pc`.
- `cargo build -p bevy_world_scale --features client --bin world_scale_client --target wasm32-unknown-unknown`: passed.
- Clean replay: matched all 30 ticks.
- Injected `accept-duplicate` fault: failed as intended with `tick=5 stable_id=100 field=position_mm expected=[1000, 0] actual=[2000, 0]`.
- `bash tools/world-scale/scripts/run_recovery_scenario.sh`: passed duplicate delivery, lost acknowledgment, reconnect, client cell unload/reload, server restart, and late join. The committed crate remained `[1000, 0]` at revision `1`; the pedestrian remained on `Detour` and continued progressing after restart.

## Paths that actually ran

| Path | Result |
| --- | --- |
| Headless native server | Ran at 30 Hz on `127.0.0.1:4200`; checkpoint and journal were recovered across a real process restart. |
| Native scripted WebSocket clients | Ran three sessions against the server: overlapping observer, reconnecting pusher, and late joiner. |
| Native graphical client | Built and linked. Direct window launch failed before app startup because the shared desktop's X authorization cookie was unavailable to the shell (`XOpenDisplayFailed`). |
| WASM graphical client | Compiled to `wasm32-unknown-unknown`; `wasm-bindgen` output was generated and Chromium fetched the HTML, JS, and 249 MiB unoptimized debug WASM. The available Chromium is forced to `--disable-gpu`; it stayed black and did not establish the WebSocket, so no successful browser-render claim is made. |

The graphical limitations are properties of this execution environment, not silently counted as successful runs. The build and serving commands above are the exact continuation path on a GPU-enabled browser or an authorized native display.
