pub mod console;
pub mod file_explorer;
pub mod launcher;
pub mod scene_hierarchy;
pub mod scene_inspector;
pub mod scene_toolbar;
pub mod viewport3d;

use bevy::prelude::*;
use bevy_camera::ClearColorConfig;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use crate::model::{
    ConsoleLog, DragDropState, EditorState, EnvironmentSettings, SceneChanged, SceneDocument,
    SceneIdCounter, SceneSelection, UndoHistory,
};

use self::{
    console::console_panel_system,
    file_explorer::{file_explorer_system, FileExplorerState},
    launcher::{launcher_system, AppMode, AppModeRes, ChosenProject, LauncherState},
    scene_hierarchy::scene_hierarchy_system,
    scene_inspector::scene_inspector_system,
    scene_toolbar::scene_toolbar_system,
    viewport3d::{
        camera_orbit_system, grid_gizmo_system, keyboard_shortcuts_system,
        node_type_gizmo_system,
        scene_sync_system, selection_gizmo_system,
        viewport_info_system, viewport_interact_system, viewport_rect_system,
        apply_viewport_rect_system, play_mode_camera_system,
        OrbitState, ViewportDragState, ViewportRect,
    },
};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app
            // App mode resource
            .init_resource::<AppModeRes>()
            // Launcher resources
            .init_resource::<LauncherState>()
            .init_resource::<ChosenProject>()
            // Core resources
            .init_resource::<SceneDocument>()
            .init_resource::<SceneSelection>()
            .init_resource::<SceneIdCounter>()
            .init_resource::<SceneChanged>()
            .init_resource::<OrbitState>()
            .init_resource::<ViewportDragState>()
            .init_resource::<ViewportRect>()
            // New resources
            .init_resource::<UndoHistory>()
            .init_resource::<EditorState>()
            .init_resource::<ConsoleLog>()
            .init_resource::<DragDropState>()
            .init_resource::<FileExplorerState>()
            .init_resource::<EnvironmentSettings>()
            // All egui systems in one chain (each checks mode internally)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    launcher_system,
                    scene_toolbar_system,
                    scene_hierarchy_system,
                    scene_inspector_system,
                    console_panel_system,
                    file_explorer_system,
                    viewport_info_system,
                    viewport_rect_system,
                )
                    .chain(),
            )
            // Spawn a camera at startup so egui has a context to render into
            .add_systems(Startup, spawn_initial_camera)
            // Update systems (check mode via resource condition)
            .add_systems(Update, (
                camera_orbit_system,
                scene_sync_system,
                selection_gizmo_system,
                node_type_gizmo_system,
                grid_gizmo_system,
                keyboard_shortcuts_system,
                viewport_interact_system,
                apply_viewport_rect_system,
                play_mode_camera_system,
            ).run_if(is_editor_mode))
            // Init editor on first frame of editor mode
            .add_systems(Update, init_editor_on_transition.run_if(is_editor_mode));
    }
}

#[derive(Component)]
struct LauncherCamera;

fn spawn_initial_camera(mut commands: Commands) {
    commands.spawn((Camera2d, LauncherCamera));
}

/// Run condition: are we in editor mode?
fn is_editor_mode(app_mode: Res<AppModeRes>) -> bool {
    app_mode.mode == AppMode::Editor
}

/// Initialize editor when transitioning from launcher
fn init_editor_on_transition(
    mut commands: Commands,
    chosen: Res<ChosenProject>,
    mut app_mode: ResMut<AppModeRes>,
    mut file_explorer: ResMut<FileExplorerState>,
    mut editor: ResMut<EditorState>,
    mut doc: ResMut<SceneDocument>,
    mut changed: ResMut<SceneChanged>,
    mut console: ResMut<ConsoleLog>,
    mut camera_q: Query<&mut Camera, With<LauncherCamera>>,
) {
    if app_mode.editor_initialized { return; }
    app_mode.editor_initialized = true;

    // Keep Camera2d active for egui, but render it on top without clearing
    if let Ok(mut cam) = camera_q.single_mut() {
        cam.order = 1;
        cam.clear_color = ClearColorConfig::None;
    }

    // Spawn 3D editor camera (renders first at order 0)
    commands.spawn((
        Camera3d::default(),
        Camera { order: 0, ..default() },
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        viewport3d::OrbitCamera,
    ));

    if let Some(project_path) = &chosen.path {
        let path = std::path::PathBuf::from(project_path);
        file_explorer.root = path.clone();
        file_explorer.current_dir = path.clone();
        file_explorer.needs_refresh = true;

        // Auto-load scene.json if it exists
        let scene_file = path.join("scene.json");
        if scene_file.exists() {
            if let Ok(json) = std::fs::read_to_string(&scene_file) {
                if let Ok(loaded) = SceneDocument::from_json(&json) {
                    *doc = loaded;
                    editor.scene_file_path = Some(scene_file.to_string_lossy().to_string());
                    changed.dirty = true;
                    console.info(format!("Loaded project: {}", path.display()));
                }
            }
        }
    }
}
