pub mod hierarchy;
pub mod inspector;
pub mod toolbar;
pub mod viewport;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPostUpdateSet};

use crate::model::{
    EditorChanged, EditorDocument, EditorIdCounter, EditorSelection, ShowExportWindow,
};

use self::{
    hierarchy::hierarchy_system,
    inspector::inspector_system,
    toolbar::toolbar_system,
    viewport::viewport_sync_system,
};
use crate::export::export_window_system;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // Ensure EguiPlugin is present (idempotent if already added in main)
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app
            // Resources
            .init_resource::<EditorDocument>()
            .init_resource::<EditorSelection>()
            .init_resource::<EditorIdCounter>()
            .init_resource::<EditorChanged>()
            .init_resource::<ShowExportWindow>()
            // Systems — run in PostUpdate before EndPass so egui context is ready
            .add_systems(
                PostUpdate,
                (
                    toolbar_system,
                    hierarchy_system,
                    inspector_system,
                    export_window_system,
                    viewport_sync_system,
                )
                    .chain()
                    .before(EguiPostUpdateSet::EndPass),
            )
            // Startup: spawn camera
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}