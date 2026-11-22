//! Native and WASM procedural world-scale client.

use bevy::prelude::*;
use bevy_world_scale::{setup_intersection, sync_presented_objects, ClientView};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .init_resource::<ClientView>()
        .add_systems(Startup, setup_intersection)
        .add_systems(Update, sync_presented_objects);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Startup, bevy_world_scale::connect_native)
        .add_systems(Update, (bevy_world_scale::receive_native, bevy_world_scale::native_controls));
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Startup, bevy_world_scale::connect_browser)
        .add_systems(Update, (bevy_world_scale::receive_browser, bevy_world_scale::browser_controls));
    app.run();
}
