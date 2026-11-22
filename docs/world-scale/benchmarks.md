# Benchmark protocol and first implementation ticket

## Existing smoke controls

From the repository root, the stock examples provide separate checks for a headless build, a desktop stress scene and the two WASM build profiles. The integrated district harness is a proposed deliverable. See the example guide for toolchain setup and how to serve the generated web example. [Web build instructions](../../examples/README.md#webgl2-and-webgpu)

```sh
cargo run --example headless --no-default-features --features bevy_log
cargo run --profile stress-test --example many_cubes
cargo run -p build-wasm-example -- --api webgl2 load_gltf
cargo run -p build-wasm-example -- --api webgpu load_gltf
```

## Fixtures to implement

| Fixture | Route and controlled workload | Failure it should reveal |
| --- | --- | --- |
| Intersection | Seed 42, 60 seconds, two observers, push and detour on adjacent cells. Start with one crate and NPC, then sweep 100/1,000/10,000 dormant props while keeping the same active set. | Lost state, expensive inactive objects, duplicate commands, stalls when many bodies wake |
| Street and Skyline | Same 50,000 repeated props, 64 source meshes, 60-second street-level and rooftop tracks. Pin projection, materials and seed. | Benefit in an occluded street that becomes a regression when most objects are visible |
| Flight and reversal | Stream coarse and detailed cells along a fast traversal, reverse during decode, loop ten times. Run cold and warm separately. | Stale activations, holes, dependency leaks, late collision, unpredictable memory |
| Night and close-up | Fixed exposure, wet road, moving lights, groomed character, shoulder turn, moving car, hard camera cut | Reflection misses, stale shadow maps, ghosting, groom popping and fill-rate limits |

## Run and decide

Register hardware model, OS, browser/driver versions, backend, adapter features, target refresh, viewport, internal resolution and device pixel ratio. Start with an integrated laptop at 720p internal/60 Hz and a phone at 720p internal/30 Hz, plus one native discrete GPU for experimental paths. Record Android and iOS separately. Run a 10-minute mobile soak and report the final three minutes. A physical-device result applies only to the profile it exercised.

Warm pipelines for 30 seconds and alternate ten paired baseline/candidate runs with balanced order. Preserve all raw runs. Analyze per-run p95 differences with a paired bootstrap interval, retaining frame sequences to diagnose p99 hitches. First run the same build twice in batches and check that median run-level p95 CPU agrees within 5%. Use a cold browser profile for first-control tests with 10 Mbps down and 80 ms RTT. Network simulation tests separately specify latency, jitter, loss, client count and observed throughput.

| Decision | Proposed gate |
| --- | --- |
| First playable district | Coarse control within 5 s cold. Count WASM, textures, scene and collision bytes. A 4 MiB transfer takes about 3.4 s at 10 Mbps before latency or startup, so report the remaining compile/decode/activation budget. |
| Streaming | Reduce p99 main-thread activation by 30% versus eager activation. Zero missing coarse cells at checkpoints. Owned bytes settle within 5% of steady state after ten loops. |
| Simulation | Reduce p95 simulation cost by 25% versus all-near updates. Preserve scripted outcome order within one 30 Hz tick and keep every durable ID across 100 tier changes. |
| Visibility | Reduce street p95 CPU submission by 20% with at most 5% skyline regression. Match object ID/depth against an unculled reference at identical LOD, excluding a declared one-pixel silhouette band. Intentionally hide one visible object to test the oracle. |
| Network and recovery | Both players see each accepted push once, including retry and rejoin. No growing tick or snapshot backlog at the declared load. Report p95 snapshot age and correction distance as client count rises. |

For every run store revision and build flags, fixture/asset/input hashes, variant, cache state, network and device settings. Frame rows carry frame ID, server tick, presentation interval, fixed-step count, CPU schedule/extract/prepare time, GPU pass times when available, active cells/bodies, estimated resident bytes, pending fetch/upload bytes and applied snapshot tick. Server rows carry tick duration, command backlog, tick lag, contact pairs, NPC tier counts, journal bytes and bytes sent per client. Associate captures by tick and camera checkpoint. Percentiles of CPU and GPU intervals cannot simply be added because work overlaps.

Report median paired improvement, its 95% interval, worst segment and the quality check side by side. Advance a change when the predeclared effect meets its target, the interval excludes zero in the favorable direction and the guard route passes. Record a null, negative or hardware-limited result with its raw run IDs. Recheck on the combined district route before adopting a different default in the fork.

## First ticket handoff

Build the headless/manual-tick intersection first, using the state and crash cases in the [simulation design](simulation.md). Deliver a fixture generator, a versioned input trace and a comparison command that prints the first differing tick, stable ID and field. The first deliberately broken build should accept a duplicate push or forget the crate after restart and turn the comparison red. Then connect native and browser clients to that server, give them the same content manifest, and record the push, detour, restart and rejoin sequence. A clean replay and both traces establish the baseline for scaling the active neighborhood. This milestone can succeed with placeholder art.
