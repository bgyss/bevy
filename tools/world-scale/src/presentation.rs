use crate::{ClientView, StableId, ViewObject};
use bevy::prelude::*;

#[derive(Component)]
pub struct PresentedObject(pub StableId);

pub fn setup_intersection(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((Sprite::from_color(Color::srgb(0.18, 0.42, 0.20), Vec2::new(420., 500.)), Transform::from_xyz(-210., 0., -2.)));
    commands.spawn((Sprite::from_color(Color::srgb(0.16, 0.36, 0.18), Vec2::new(420., 500.)), Transform::from_xyz(210., 0., -2.)));
    commands.spawn((Sprite::from_color(Color::srgb(0.14, 0.14, 0.16), Vec2::new(840., 180.)), Transform::from_xyz(0., 0., -1.)));
    commands.spawn((Sprite::from_color(Color::srgb(0.14, 0.14, 0.16), Vec2::new(180., 500.)), Transform::from_xyz(0., 0., -1.)));
    for x in [-60., -30., 0., 30., 60.] {
        commands.spawn((Sprite::from_color(Color::WHITE, Vec2::new(14., 110.)), Transform::from_xyz(x, 0., 0.)));
    }
    commands.spawn((Sprite::from_color(Color::srgb(0.65, 0.34, 0.12), Vec2::splat(48.)), Transform::from_xyz(0., 0., 2.), PresentedObject(StableId(100))));
    commands.spawn((Sprite::from_color(Color::srgb(0.95, 0.75, 0.20), Vec2::new(30., 48.)), Transform::from_xyz(150., -100., 2.), PresentedObject(StableId(200))));
}

pub fn sync_presented_objects(view: Res<ClientView>, mut objects: Query<(&PresentedObject, &mut Transform)>) {
    if !view.is_changed() { return; }
    for (presented, mut transform) in &mut objects {
        let position = match view.object(presented.0) {
            Some(ViewObject::Crate(state)) => state.position_mm,
            Some(ViewObject::Pedestrian(state)) => state.position_mm,
            None => continue,
        };
        transform.translation.x = position[0] as f32 * 0.1 - 100.;
        transform.translation.y = position[1] as f32 * 0.1;
    }
}
