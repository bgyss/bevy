use crate::{TickRecord, WorldState, SCHEMA_VERSION};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

pub struct DurableStore {
    directory: PathBuf,
    journal: File,
}

impl DurableStore {
    pub fn open(path: impl AsRef<Path>, initial: WorldState) -> io::Result<Self> {
        let directory = path.as_ref().to_path_buf();
        fs::create_dir_all(&directory)?;
        let checkpoint = directory.join("checkpoint.json");
        if !checkpoint.exists() {
            write_checkpoint(&directory, &initial)?;
        }
        let journal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("journal.jsonl"))?;
        Ok(Self { directory, journal })
    }

    pub fn commit_tick(&mut self, record: TickRecord, state: &WorldState) -> io::Result<()> {
        if record.tick != state.tick {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "tick record and state disagree",
            ));
        }
        serde_json::to_writer(&mut self.journal, state).map_err(io::Error::other)?;
        self.journal.write_all(b"\n")?;
        self.journal.flush()?;
        self.journal.sync_data()
    }

    pub fn checkpoint(&mut self, state: &WorldState) -> io::Result<()> {
        write_checkpoint(&self.directory, state)?;
        self.journal = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.directory.join("journal.jsonl"))?;
        self.journal.sync_data()
    }

    pub fn recover(path: impl AsRef<Path>) -> io::Result<WorldState> {
        let directory = path.as_ref();
        let checkpoint = File::open(directory.join("checkpoint.json"))?;
        let mut state: WorldState =
            serde_json::from_reader(checkpoint).map_err(io::Error::other)?;
        if state.version != SCHEMA_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported checkpoint version {}", state.version),
            ));
        }
        let journal_path = directory.join("journal.jsonl");
        if let Ok(journal) = File::open(journal_path) {
            for line in BufReader::new(journal).lines() {
                let line = line?;
                match serde_json::from_str::<WorldState>(&line) {
                    Ok(candidate)
                        if candidate.version == SCHEMA_VERSION && candidate.tick > state.tick =>
                    {
                        state = candidate
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
        Ok(state)
    }
}

fn write_checkpoint(directory: &Path, state: &WorldState) -> io::Result<()> {
    let temporary = directory.join("checkpoint.json.tmp");
    let destination = directory.join("checkpoint.json");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer(&mut file, state).map_err(io::Error::other)?;
    file.flush()?;
    file.sync_data()?;
    fs::rename(temporary, destination)
}

#[cfg(test)]
mod tests {
    use super::DurableStore;
    use crate::{Command, Fixture, Simulation, StableId};
    use std::{
        fs::OpenOptions,
        io::Write,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn state_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bevy-world-scale-{name}-{nonce}"))
    }

    #[test]
    fn committed_tick_recovers_crate_pedestrian_and_deduplication() {
        let path = state_dir("recover");
        let mut simulation = Simulation::from_fixture(Fixture::intersection(42));
        let mut store = DurableStore::open(&path, simulation.state().clone()).unwrap();
        for _ in 0..4 {
            store
                .commit_tick(simulation.tick(&[]), simulation.state())
                .unwrap();
        }
        let push = Command::push(7, 1, 5, StableId(100));
        store
            .commit_tick(simulation.tick(&[push.clone()]), simulation.state())
            .unwrap();
        drop(store);

        let recovered = DurableStore::recover(&path).unwrap();
        let pedestrian_position = recovered.pedestrian.position_mm;
        let mut restarted = Simulation::from_state(recovered);
        restarted.tick(&[push]);

        assert_eq!(restarted.state().crate_state.position_mm, [1_000, 0]);
        assert_eq!(restarted.state().crate_state.revision, 1);
        assert_eq!(restarted.state().accepted_commands.len(), 1);
        assert_ne!(pedestrian_position, [1_500, -1_000]);
    }

    #[test]
    fn truncated_journal_tail_is_ignored() {
        let path = state_dir("tail");
        let mut simulation = Simulation::from_fixture(Fixture::intersection(42));
        let mut store = DurableStore::open(&path, simulation.state().clone()).unwrap();
        store
            .commit_tick(simulation.tick(&[]), simulation.state())
            .unwrap();
        drop(store);
        OpenOptions::new()
            .append(true)
            .open(path.join("journal.jsonl"))
            .unwrap()
            .write_all(b"{partial")
            .unwrap();

        let recovered = DurableStore::recover(&path).unwrap();
        assert_eq!(recovered.tick, 1);
    }
}
