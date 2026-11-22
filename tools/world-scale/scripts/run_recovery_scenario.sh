#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/../../.." && pwd)
target_dir=${CARGO_TARGET_DIR:-/workspace/bevy-target}
state_dir=$(mktemp -d /tmp/bevy-world-scale-scenario.XXXXXX)
server_log="$state_dir/server.log"
observer_log="$state_dir/observer.log"
server_pid=""

cleanup() {
  if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT

start_server() {
  "$target_dir/debug/world_scale_server" --state "$state_dir" >"$server_log" 2>&1 &
  server_pid=$!
  for _ in $(seq 1 100); do
    if [[ -s "$server_log" ]] && rg -q 'listening' "$server_log"; then return; fi
    sleep 0.05
  done
  echo "server did not become ready" >&2
  exit 1
}

cd "$repo_root"
cargo build -p bevy_world_scale --bins
start_server

"$target_dir/debug/world_scale_probe" --session 22 --wait-change >"$observer_log" &
observer_pid=$!
"$target_dir/debug/world_scale_probe" --session 11 --push --duplicate --drop-ack
"$target_dir/debug/world_scale_probe" --session 11 --push --duplicate --expect-committed
wait "$observer_pid"
rg -q 'push_count=1 crate_revision=1 .*route=Detour' "$observer_log"
"$target_dir/debug/world_scale_probe" --session 11 --unload-reload --expect-committed

kill "$server_pid"
wait "$server_pid" 2>/dev/null || true
server_pid=""
start_server
"$target_dir/debug/world_scale_probe" --session 33 --expect-committed

echo "PASS duplicate delivery, lost ack, reconnect, unload/reload, restart, late join"
echo "PASS push_count=1 crate_revision=1 crate_position=[1000,0] pedestrian_route=Detour"
