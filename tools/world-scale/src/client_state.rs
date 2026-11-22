use crate::{CellId, CrateState, PedestrianState, StableId, ViewSnapshot};
use bevy::prelude::Resource;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum ViewObject {
    Crate(CrateState),
    Pedestrian(PedestrianState),
}

impl ViewObject {
    fn cell(&self) -> CellId {
        match self {
            Self::Crate(state) => state.cell,
            Self::Pedestrian(state) => state.cell,
        }
    }
}

#[derive(Default, Resource)]
pub struct ClientView {
    objects: BTreeMap<StableId, ViewObject>,
    pub last_tick: u64,
}

impl ClientView {
    pub fn apply(&mut self, snapshot: ViewSnapshot) {
        if snapshot.complete {
            self.objects
                .retain(|_, object| !snapshot.cells.contains(&object.cell()));
        }
        if let Some(state) = snapshot.crate_state {
            self.objects
                .insert(state.stable_id, ViewObject::Crate(state));
        }
        if let Some(state) = snapshot.pedestrian {
            self.objects
                .insert(state.stable_id, ViewObject::Pedestrian(state));
        }
        self.last_tick = snapshot.tick;
    }
    pub fn unload(&mut self, cell: CellId) {
        self.objects.retain(|_, object| object.cell() != cell);
    }
    pub fn objects(&self) -> &BTreeMap<StableId, ViewObject> {
        &self.objects
    }
    pub fn object(&self, id: StableId) -> Option<&ViewObject> {
        self.objects.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientView;
    use crate::{CellId, Fixture, ServerCore};

    #[test]
    fn complete_snapshot_rebuilds_unloaded_cell_from_stable_ids() {
        let path = std::env::temp_dir().join("bevy-world-scale-client-view");
        let server = ServerCore::fresh(&path, Fixture::intersection(42)).unwrap();
        let mut view = ClientView::default();
        view.apply(server.snapshot(&[CellId(0), CellId(1)], true));
        assert_eq!(view.objects().len(), 2);
        view.unload(CellId(1));
        assert_eq!(view.objects().len(), 1);
        view.apply(server.snapshot(&[CellId(1)], true));
        assert_eq!(view.objects().len(), 2);
    }
}
