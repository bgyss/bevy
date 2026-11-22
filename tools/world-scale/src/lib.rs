//! Fork-local deterministic world-scale intersection experiment.
#![allow(
    missing_docs,
    reason = "Fork-local experiment without a public API contract"
)]

extern crate alloc;

mod client_state;
#[cfg(feature = "client")]
mod client_transport;
mod fixture;
mod model;
mod persistence;
#[cfg(feature = "client")]
mod presentation;
mod protocol;
mod replay;
mod server;
mod simulation;

pub use client_state::*;
#[cfg(feature = "client")]
pub use client_transport::*;
pub use fixture::*;
pub use model::*;
pub use persistence::*;
#[cfg(feature = "client")]
pub use presentation::*;
pub use protocol::*;
pub use replay::*;
pub use server::*;
pub use simulation::*;

#[cfg(test)]
mod tests {
    use crate::{compare_runs, Fixture, Replay, Simulation, StableId};

    #[test]
    fn intersection_has_stable_fixture_ids() {
        let fixture = Fixture::intersection(42);
        assert_eq!(fixture.crate_id, StableId(100));
        assert_eq!(fixture.pedestrian_id, StableId(200));
        assert_eq!(fixture.cells.len(), 2);
    }

    #[test]
    fn duplicate_push_moves_crate_once_and_starts_detour() {
        let mut simulation = Simulation::from_fixture(Fixture::intersection(42));
        let command = crate::Command::push(7, 1, 5, StableId(100));
        for _ in 0..4 {
            simulation.tick(&[]);
        }
        simulation.tick(&[command.clone(), command]);

        assert_eq!(simulation.state().crate_state.position_mm, [1_000, 0]);
        assert_eq!(simulation.state().crate_state.revision, 1);
        assert_eq!(simulation.state().pedestrian.route, crate::Route::Detour);
        assert_eq!(simulation.state().accepted_commands.len(), 1);
    }

    #[test]
    fn comparator_names_first_tick_object_and_field() {
        let replay = Replay::intersection_v1();
        let expected = crate::run_replay(&replay, false).unwrap();
        let actual = crate::run_replay(&replay, true).unwrap();
        let difference = compare_runs(&expected, &actual).unwrap_err();

        assert_eq!(difference.tick, 5);
        assert_eq!(difference.stable_id, StableId(100));
        assert_eq!(difference.field, "position_mm");
    }

    #[test]
    fn replay_version_mismatch_is_rejected() {
        let mut replay = Replay::intersection_v1();
        replay.version = 99;
        let error = crate::run_replay(&replay, false).unwrap_err();
        assert!(error.to_string().contains("unsupported replay version 99"));
    }
}
