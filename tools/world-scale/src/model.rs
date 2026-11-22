use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct StableId(pub u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CellId(pub i32);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ObjectKind { Crate, Pedestrian, Crossing, PlayerSpawn }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObjectRecord {
    pub stable_id: StableId,
    pub kind: ObjectKind,
    pub cell: CellId,
    pub position_mm: [i32; 2],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CellManifest {
    pub version: u32,
    pub seed: u64,
    pub cells: Vec<CellId>,
    pub objects: Vec<ObjectRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Route { Direct, Detour }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CrateState {
    pub stable_id: StableId,
    pub cell: CellId,
    pub position_mm: [i32; 2],
    pub velocity_mm_per_tick: [i32; 2],
    pub ownership_epoch: u64,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PedestrianState {
    pub stable_id: StableId,
    pub cell: CellId,
    pub position_mm: [i32; 2],
    pub route: Route,
    pub route_progress_mm: u32,
    pub next_decision_tick: u64,
    pub revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcceptedCommand {
    pub session_id: u64,
    pub sequence: u64,
    pub accepted_tick: u64,
    pub target: StableId,
    pub resulting_revision: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CommandKind { Push { impulse_mm: [i32; 2] } }

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Command {
    pub session_id: u64,
    pub sequence: u64,
    pub tick: u64,
    pub target: StableId,
    pub kind: CommandKind,
}

impl Command {
    pub fn push(session_id: u64, sequence: u64, tick: u64, target: StableId) -> Self {
        Self { session_id, sequence, tick, target, kind: CommandKind::Push { impulse_mm: [1_000, 0] } }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Resource, Serialize)]
pub struct WorldState {
    pub version: u32,
    pub tick: u64,
    pub crate_state: CrateState,
    pub pedestrian: PedestrianState,
    pub accepted_commands: BTreeMap<String, AcceptedCommand>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TickRecord {
    pub tick: u64,
    pub crate_state: CrateState,
    pub pedestrian: PedestrianState,
    pub accepted: Vec<AcceptedCommand>,
}
