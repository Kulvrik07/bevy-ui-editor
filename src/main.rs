use bevy::prelude::*;

use bevy_rapier3d::prelude::*;

mod editor;
mod export;
mod model;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy 3D Editor".to_string(),
                resolution: (1400u32, 900u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default()) // Show colliders
        .add_plugins(editor::EditorPlugin)
        .run();
}
