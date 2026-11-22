# A persistent city in Bevy

Start with **one intersection, two players and one movable crate**. A player pushes the crate into a crossing. A pedestrian takes a detour. The other player sees both events. Both leave, the server restarts, and a returning player finds the crate where it settled and the pedestrian farther along the route. A low-end browser should tell the same story with simpler geometry, shadows and hair. This scene puts a specific meaning on “interactive” and exposes failures that a million stationary meshes would miss.

This is a research plan for the fork at `8c3f96155` (Cargo version `0.20.0-dev`). GTA VI supplies a visual and experiential reference. Visible reactions alone cannot establish the number of active rigid bodies, RAGE's solver, or a browser performance target. The useful question for this fork is how far the *radius of reliable interaction* and the *density visible beyond it* can grow on a named device before a budget breaks.

## From the earlier research to this fork

The supplied *Research Plan for Scalable Web Games with three.js and Babylon.js* contributes paired Street/Skyline routes, cold traversal, mobile budgets and image checks. Use the same fixtures across Bevy builds. Its JS adapter comparisons and upstream handoff tickets belonged to the earlier project. *Rendering and Simulation Improvements for Game Engines* contributes separate budgets for activation, simulation, geometry, lighting and grooms. Check its release-level observations against this checkout before picking an implementation. The distributed authority, recovery and two-client experiments below extend those earlier benchmarks.

## What to build, in order

| Milestone | Change in the fork | Evidence needed to proceed |
| --- | --- | --- |
| 1. Intersection replay | Prototype a shared simulation plugin in a fork-local experiment with a manual fixed-tick driver, procedural geometry and stable object IDs. Start with one native process, then a separate headless server and native and WASM clients. | Both observers see the same accepted interactions. Late join, retry, unload and restart pass the [recovery experiment](simulation.md). |
| 2. A district larger than memory | Give static content a versioned cell manifest and budget activation. Add blocks while holding the visible route and interaction radius constant. | Resident bytes plateau on repeated reversal. A player can traverse on coarse geometry during a delayed detail fetch. |
| 3. Active neighborhoods | Introduce sleeping bodies and tiered NPC schedules. Compare against continuously simulated actors with the same inputs. | Critical contacts and detours survive tier changes. Tick cost falls on a populated street. |
| 4. A visually dense district | Compare stock PBR against spatial batching. Evaluate shadows, hair and reconstruction individually on saved camera tracks. | A variant meets its [effect-specific quality check](presentation.md) and improves its declared budget on physical hardware. |
| 5. Multiple authorities | Split the same district across two workers once one measured worker saturates. Test border crossing under load and worker loss. | Exactly one owner accepts a command. Recovery and cross-border contacts pass the same tests as the single worker. |
| 6. Combined traversal | Drive through traffic at dusk, enter a crowded shop, reverse through the streaming boundary and reconnect. | The slowest segment passes both interaction and presentation checks on each claimed profile. Record unsupported profiles explicitly. |

Milestones 2 and 3 can share the intersection fixture. Lighting and groom work can use the same camera harness independently once repeatability is established. Keep experiments as optional plugins with stock Bevy controls. Move a change into an engine crate when a trace identifies a cost that the public API cannot address, such as upload scheduling inside render-asset preparation.

## Existing controls in this checkout

| Question | Entry point |
| --- | --- |
| Can the server tick without a GPU? | [Headless runner](../../examples/app/headless.rs), [fixed schedule](../../examples/ecs/fixed_timestep.rs) |
| What is the stock cost of actors, lights and repeated geometry? | [many foxes](../../examples/stress_tests/many_foxes.rs), [many lights](../../examples/stress_tests/many_lights.rs), [many cubes](../../examples/stress_tests/many_cubes.rs) |
| What does the current renderer already eliminate? | [Visibility ranges](../../examples/3d/visibility_range.rs), [occlusion culling](../../examples/3d/occlusion_culling.rs), [GPU batch preparation](../../crates/bevy_render/src/batching/) |
| Where does loading become rendering work? | [Asset server](../../crates/bevy_asset/src/server/), [render asset preparation](../../crates/bevy_render/src/render_asset.rs) |
| How should we collect traces and build browser controls? | [Profiling guide](../profiling.md), [WASM example instructions](../../examples/README.md#webgl2-and-webgpu), [benchmark protocol](benchmarks.md) |

The first implementation ticket ends with a recorded replay, a server checkpoint and two client event traces for the intersection. The following ticket changes one variable, such as sleep policy, against that recording. All thresholds here are proposed decision criteria. No performance or parity result has been measured.

The first runnable milestone now lives in [`tools/world-scale`](../../tools/world-scale/README.md), including exact server/client/replay commands and the recorded execution evidence.
