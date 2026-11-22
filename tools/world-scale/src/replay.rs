use crate::{Command, Fixture, Simulation, StableId, TickRecord, SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Replay { pub version: u32, pub fixture_seed: u64, pub end_tick: u64, pub commands: Vec<Command> }
impl Replay {
    pub fn intersection_v1() -> Self {
        let push = Command::push(7, 1, 5, StableId(100));
        Self { version: SCHEMA_VERSION, fixture_seed: 42, end_tick: 30, commands: vec![push.clone(), push] }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayError(pub String);
impl fmt::Display for ReplayError { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) } }
impl Error for ReplayError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Difference { pub tick: u64, pub stable_id: StableId, pub field: &'static str, pub expected: String, pub actual: String }

pub fn run_replay(replay: &Replay, accept_duplicate_fault: bool) -> Result<Vec<TickRecord>, ReplayError> {
    if replay.version != SCHEMA_VERSION { return Err(ReplayError(format!("unsupported replay version {} (expected {SCHEMA_VERSION})", replay.version))); }
    let mut simulation = Simulation::with_fault(Fixture::intersection(replay.fixture_seed), accept_duplicate_fault);
    let mut records = Vec::new();
    for tick in 1..=replay.end_tick {
        let commands = replay.commands.iter().filter(|command| command.tick == tick).cloned().collect::<Vec<_>>();
        records.push(simulation.tick(&commands));
    }
    Ok(records)
}

pub fn compare_runs(expected: &[TickRecord], actual: &[TickRecord]) -> Result<(), Difference> {
    for (expected, actual) in expected.iter().zip(actual) {
        if expected.crate_state.position_mm != actual.crate_state.position_mm { return Err(Difference { tick: expected.tick, stable_id: expected.crate_state.stable_id, field: "position_mm", expected: format!("{:?}", expected.crate_state.position_mm), actual: format!("{:?}", actual.crate_state.position_mm) }); }
        if expected.crate_state.revision != actual.crate_state.revision { return Err(Difference { tick: expected.tick, stable_id: expected.crate_state.stable_id, field: "revision", expected: expected.crate_state.revision.to_string(), actual: actual.crate_state.revision.to_string() }); }
        if expected.pedestrian.route != actual.pedestrian.route { return Err(Difference { tick: expected.tick, stable_id: expected.pedestrian.stable_id, field: "route", expected: format!("{:?}", expected.pedestrian.route), actual: format!("{:?}", actual.pedestrian.route) }); }
        if expected.pedestrian.position_mm != actual.pedestrian.position_mm { return Err(Difference { tick: expected.tick, stable_id: expected.pedestrian.stable_id, field: "position_mm", expected: format!("{:?}", expected.pedestrian.position_mm), actual: format!("{:?}", actual.pedestrian.position_mm) }); }
    }
    if expected.len() != actual.len() { return Err(Difference { tick: expected.len().min(actual.len()) as u64 + 1, stable_id: StableId(0), field: "record_count", expected: expected.len().to_string(), actual: actual.len().to_string() }); }
    Ok(())
}
