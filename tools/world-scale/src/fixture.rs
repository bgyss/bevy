use crate::{CellId, CellManifest, CrateState, ObjectKind, ObjectRecord, PedestrianState, Route, SCHEMA_VERSION, StableId, WorldState};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Fixture {
    pub manifest: CellManifest,
    pub cells: Vec<CellId>,
    pub crate_id: StableId,
    pub pedestrian_id: StableId,
    pub crossing_id: StableId,
}

impl Fixture {
    pub fn intersection(seed: u64) -> Self {
        let cells = vec![CellId(0), CellId(1)];
        let objects = vec![
            ObjectRecord { stable_id: StableId(100), kind: ObjectKind::Crate, cell: CellId(0), position_mm: [0, 0] },
            ObjectRecord { stable_id: StableId(200), kind: ObjectKind::Pedestrian, cell: CellId(1), position_mm: [1_500, -1_000] },
            ObjectRecord { stable_id: StableId(300), kind: ObjectKind::Crossing, cell: CellId(0), position_mm: [1_000, 0] },
            ObjectRecord { stable_id: StableId(400), kind: ObjectKind::PlayerSpawn, cell: CellId(0), position_mm: [-1_000, 0] },
            ObjectRecord { stable_id: StableId(401), kind: ObjectKind::PlayerSpawn, cell: CellId(1), position_mm: [2_000, 0] },
        ];
        let manifest = CellManifest { version: SCHEMA_VERSION, seed, cells: cells.clone(), objects };
        Self { manifest, cells, crate_id: StableId(100), pedestrian_id: StableId(200), crossing_id: StableId(300) }
    }

    pub fn initial_state(&self) -> WorldState {
        WorldState {
            version: SCHEMA_VERSION,
            tick: 0,
            crate_state: CrateState { stable_id: self.crate_id, cell: CellId(0), position_mm: [0, 0], velocity_mm_per_tick: [0, 0], ownership_epoch: 1, revision: 0 },
            pedestrian: PedestrianState { stable_id: self.pedestrian_id, cell: CellId(1), position_mm: [1_500, -1_000], route: Route::Direct, route_progress_mm: 0, next_decision_tick: 5, revision: 0 },
            accepted_commands: BTreeMap::new(),
        }
    }
}
