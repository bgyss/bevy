use crate::{AcceptedCommand, Command, CommandKind, Fixture, Route, TickRecord, WorldState};
use bevy::{prelude::*, time::Fixed};
use std::time::Duration;

#[derive(Resource)]
pub struct Simulation { state: WorldState, accept_duplicates: bool }

impl Simulation {
    pub fn from_fixture(fixture: Fixture) -> Self { Self { state: fixture.initial_state(), accept_duplicates: false } }
    pub fn with_fault(fixture: Fixture, accept_duplicates: bool) -> Self { Self { state: fixture.initial_state(), accept_duplicates } }
    pub fn from_state(state: WorldState) -> Self { Self { state, accept_duplicates: false } }
    pub fn state(&self) -> &WorldState { &self.state }

    pub fn tick(&mut self, commands: &[Command]) -> TickRecord {
        self.state.tick += 1;
        let tick = self.state.tick;
        let mut accepted = Vec::new();
        let mut ordered = commands.to_vec();
        ordered.sort_by_key(|command| (command.tick, command.session_id, command.sequence));
        for command in ordered.into_iter().filter(|command| command.tick <= tick) {
            let key = (command.session_id, command.sequence);
            if !self.accept_duplicates && self.state.accepted_commands.contains_key(&key) { continue; }
            if command.target != self.state.crate_state.stable_id { continue; }
            let CommandKind::Push { impulse_mm } = command.kind;
            self.state.crate_state.position_mm[0] += impulse_mm[0];
            self.state.crate_state.position_mm[1] += impulse_mm[1];
            self.state.crate_state.revision += 1;
            let outcome = AcceptedCommand { session_id: command.session_id, sequence: command.sequence, accepted_tick: tick, target: command.target, resulting_revision: self.state.crate_state.revision };
            self.state.accepted_commands.insert(key, outcome.clone());
            accepted.push(outcome);
        }
        if self.state.crate_state.position_mm == [1_000, 0] && self.state.pedestrian.route == Route::Direct {
            self.state.pedestrian.route = Route::Detour;
            self.state.pedestrian.revision += 1;
        }
        self.state.pedestrian.route_progress_mm += if self.state.pedestrian.route == Route::Detour { 80 } else { 100 };
        self.state.pedestrian.position_mm[0] += 80;
        if self.state.pedestrian.route == Route::Detour { self.state.pedestrian.position_mm[1] += 40; }
        self.state.pedestrian.revision += 1;
        TickRecord { tick, crate_state: self.state.crate_state.clone(), pedestrian: self.state.pedestrian.clone(), accepted }
    }
}

pub struct WorldScalePlugin;
impl Plugin for WorldScalePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 30.0)));
    }
}
