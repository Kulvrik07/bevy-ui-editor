use bevy::prelude::*;

mod editor;
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
        .add_plugins(editor::EditorPlugin)
        .run();
}
