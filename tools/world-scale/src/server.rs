use crate::{CellId, Command, DurableStore, Fixture, ProtocolError, Simulation, TickRecord, ViewSnapshot, WorldState, PROTOCOL_VERSION};
use std::{io, path::{Path, PathBuf}};

pub struct ServerCore {
    simulation: Simulation,
    store: DurableStore,
    state_path: PathBuf,
    fixture: Fixture,
}

impl ServerCore {
    pub fn fresh(path: impl AsRef<Path>, fixture: Fixture) -> io::Result<Self> {
        let state_path = path.as_ref().to_path_buf();
        let simulation = Simulation::from_fixture(fixture.clone());
        let store = DurableStore::open(&state_path, simulation.state().clone())?;
        Ok(Self { simulation, store, state_path, fixture })
    }

    pub fn recover(path: impl AsRef<Path>) -> io::Result<Self> {
        let state_path = path.as_ref().to_path_buf();
        let state = DurableStore::recover(&state_path)?;
        let fixture = Fixture::intersection(42);
        let store = DurableStore::open(&state_path, state.clone())?;
        Ok(Self { simulation: Simulation::from_state(state), store, state_path, fixture })
    }

    pub fn tick(&mut self, commands: &[Command]) -> io::Result<TickRecord> {
        let record = self.simulation.tick(commands);
        self.store.commit_tick(record.clone(), self.simulation.state())?;
        Ok(record)
    }

    pub fn snapshot(&self, cells: &[CellId], complete: bool) -> ViewSnapshot {
        let state = self.simulation.state();
        ViewSnapshot {
            tick: state.tick,
            complete,
            cells: cells.to_vec(),
            crate_state: cells.contains(&state.crate_state.cell).then(|| state.crate_state.clone()),
            pedestrian: cells.contains(&state.pedestrian.cell).then(|| state.pedestrian.clone()),
        }
    }

    pub fn state(&self) -> &WorldState { self.simulation.state() }
    pub fn fixture(&self) -> &Fixture { &self.fixture }
    pub fn state_path(&self) -> &Path { &self.state_path }
    pub fn validate_protocol(version: u32) -> Result<(), ProtocolError> {
        if version == PROTOCOL_VERSION { Ok(()) } else { Err(ProtocolError::UnsupportedVersion(version)) }
    }
}

#[cfg(test)]
mod tests {
    use super::ServerCore;
    use crate::{CellId, Command, Fixture, ProtocolError, StableId, PROTOCOL_VERSION};

    #[test]
    fn both_observers_and_late_join_see_committed_push_and_detour() {
        let path = std::env::temp_dir().join(format!("bevy-world-scale-server-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let mut server = ServerCore::fresh(&path, Fixture::intersection(42)).unwrap();
        for _ in 0..4 { server.tick(&[]).unwrap(); }
        let push = Command::push(7, 1, 5, StableId(100));
        server.tick(&[push.clone(), push]).unwrap();
        let observer_a = server.snapshot(&[CellId(0), CellId(1)], true);
        let observer_b = server.snapshot(&[CellId(0), CellId(1)], true);
        let late_join = ServerCore::recover(&path).unwrap().snapshot(&[CellId(0), CellId(1)], true);

        for snapshot in [observer_a, observer_b, late_join] {
            assert_eq!(snapshot.crate_state.unwrap().position_mm, [1_000, 0]);
            assert_eq!(snapshot.pedestrian.unwrap().route, crate::Route::Detour);
        }
    }

    #[test]
    fn incompatible_client_is_rejected() {
        let error = ServerCore::validate_protocol(PROTOCOL_VERSION + 1).unwrap_err();
        assert_eq!(error, ProtocolError::UnsupportedVersion(PROTOCOL_VERSION + 1));
    }
}
