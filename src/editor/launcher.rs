use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Frame, Margin, RichText, ScrollArea};
use bevy_egui::EguiContexts;

// ─── App mode (simple resource, works in any schedule) ────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Launcher,
    Editor,
}

#[derive(Resource)]
pub struct AppModeRes {
    pub mode: AppMode,
    pub editor_initialized: bool,
}

impl Default for AppModeRes {
    fn default() -> Self {
        AppModeRes {
            mode: AppMode::Launcher,
            editor_initialized: false,
        }
    }
}

#[derive(Resource)]
pub struct LauncherState {
    pub projects: Vec<ProjectEntry>,
    pub new_project_name: String,
    pub new_project_path: String,
    pub show_create: bool,
    pub error_msg: Option<String>,
}

impl Default for LauncherState {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        let projects_file = PathBuf::from(&home).join(".bevy_editor_projects.json");
        let projects = load_projects(&projects_file);

        LauncherState {
            projects,
            new_project_name: String::new(),
            new_project_path: format!("{home}/Projects"),
            show_create: false,
            error_msg: None,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: String,
}

// ─── Project persistence ──────────────────────────────────────────────────────

fn projects_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    PathBuf::from(home).join(".bevy_editor_projects.json")
}

fn load_projects(path: &PathBuf) -> Vec<ProjectEntry> {
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_projects(projects: &[ProjectEntry]) {
    let path = projects_file_path();
    if let Ok(json) = serde_json::to_string_pretty(projects) {
        let _ = std::fs::write(path, json);
    }
}

// ─── Chosen project resource ─────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ChosenProject {
    pub path: Option<String>,
}

// ─── Launcher system ──────────────────────────────────────────────────────────

pub fn launcher_system(
    mut ctx: EguiContexts,
    mut state: ResMut<LauncherState>,
    mut chosen: ResMut<ChosenProject>,
    mut app_mode: ResMut<AppModeRes>,
) {
    if app_mode.mode != AppMode::Launcher { return; }

    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    egui::CentralPanel::default()
        .frame(Frame::new()
            .fill(Color32::from_rgb(22, 22, 26))
            .inner_margin(Margin::same(20))
        )
        .show(egui_ctx, |ui| {
            ui.add_space(40.0);

            // Title
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("🎮 Bevy 3D Editor")
                        .size(32.0)
                        .strong()
                        .color(Color32::from_rgb(100, 170, 255)),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Create or open a project to get started")
                        .size(14.0)
                        .color(Color32::from_rgb(140, 140, 150)),
                );
            });

            ui.add_space(30.0);

            // Content area
            let panel_width = 600.0_f32.min(ui.available_width() - 40.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(panel_width);

                // ── Action buttons ────────────────────────────────────
                ui.horizontal(|ui| {
                    if ui.button(
                        RichText::new("➕ New Project").size(14.0).color(Color32::from_rgb(80, 220, 80)),
                    ).clicked() {
                        state.show_create = !state.show_create;
                        state.error_msg = None;
                    }
                });

                // ── Create project form ───────────────────────────────
                if state.show_create {
                    ui.add_space(10.0);
                    Frame::new()
                        .fill(Color32::from_rgb(35, 35, 40))
                        .corner_radius(6.0)
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Create New Project").strong().color(Color32::from_rgb(200, 200, 200)));
                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Name:").color(Color32::from_rgb(160, 160, 160)));
                                ui.add_space(8.0);
                                ui.add(egui::TextEdit::singleline(&mut state.new_project_name).desired_width(300.0));
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Path:  ").color(Color32::from_rgb(160, 160, 160)));
                                ui.add_space(8.0);
                                ui.add(egui::TextEdit::singleline(&mut state.new_project_path).desired_width(300.0));
                            });

                            ui.add_space(8.0);

                            if let Some(err) = &state.error_msg {
                                ui.label(RichText::new(err).color(Color32::from_rgb(230, 80, 80)).size(12.0));
                                ui.add_space(4.0);
                            }

                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("Create").color(Color32::from_rgb(80, 220, 80))).clicked() {
                                    let name = state.new_project_name.trim().to_string();
                                    let base = state.new_project_path.trim().to_string();

                                    if name.is_empty() {
                                        state.error_msg = Some("Project name cannot be empty.".to_string());
                                    } else {
                                        let project_dir = PathBuf::from(&base).join(&name);
                                        match create_project_dir(&project_dir) {
                                            Ok(()) => {
                                                let entry = ProjectEntry {
                                                    name: name.clone(),
                                                    path: project_dir.to_string_lossy().to_string(),
                                                };
                                                state.projects.push(entry);
                                                save_projects(&state.projects);
                                                // Open the project
                                                chosen.path = Some(project_dir.to_string_lossy().to_string());
                                                app_mode.mode = AppMode::Editor;
                                            }
                                            Err(e) => {
                                                state.error_msg = Some(format!("Failed to create project: {e}"));
                                            }
                                        }
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    state.show_create = false;
                                }
                            });
                        });
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                // ── Recent projects ───────────────────────────────────
                ui.label(RichText::new("Recent Projects").strong().size(16.0).color(Color32::from_rgb(200, 200, 200)));
                ui.add_space(8.0);

                if state.projects.is_empty() {
                    ui.label(
                        RichText::new("No projects yet. Create one to get started!")
                            .color(Color32::from_rgb(120, 120, 130))
                            .italics(),
                    );
                } else {
                    let mut open_idx: Option<usize> = None;
                    let mut remove_idx: Option<usize> = None;

                    ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for (idx, project) in state.projects.iter().enumerate() {
                            let exists = PathBuf::from(&project.path).exists();

                            Frame::new()
                                .fill(Color32::from_rgb(30, 30, 35))
                                .corner_radius(4.0)
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&project.name)
                                                    .strong()
                                                    .size(14.0)
                                                    .color(if exists {
                                                        Color32::from_rgb(200, 200, 220)
                                                    } else {
                                                        Color32::from_rgb(150, 100, 100)
                                                    }),
                                            );
                                            ui.label(
                                                RichText::new(&project.path)
                                                    .size(11.0)
                                                    .color(Color32::from_rgb(120, 120, 130)),
                                            );
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("✕").on_hover_text("Remove from list").clicked() {
                                                remove_idx = Some(idx);
                                            }
                                            ui.add_space(8.0);
                                            if exists {
                                                if ui.button(
                                                    RichText::new("Open").color(Color32::from_rgb(100, 180, 255)),
                                                ).clicked() {
                                                    open_idx = Some(idx);
                                                }
                                            } else {
                                                ui.label(RichText::new("Missing").color(Color32::from_rgb(180, 80, 80)).size(11.0));
                                            }
                                        });
                                    });
                                });
                            ui.add_space(4.0);
                        }
                    });

                    if let Some(idx) = open_idx {
                        chosen.path = Some(state.projects[idx].path.clone());
                        app_mode.mode = AppMode::Editor;
                    }
                    if let Some(idx) = remove_idx {
                        state.projects.remove(idx);
                        save_projects(&state.projects);
                    }
                }
            });
        });
}

// ─── Create project directory ─────────────────────────────────────────────────

fn create_project_dir(path: &PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;

    // Create a default scene file
    let scene_path = path.join("scene.json");
    if !scene_path.exists() {
        let default_scene = r#"{
  "nodes": [
    {
      "id": 1,
      "name": "Ground",
      "kind": {"Mesh": "Plane"},
      "children": [],
      "translation": [0.0, 0.0, 0.0],
      "rotation_euler": [0.0, 0.0, 0.0],
      "scale": [5.0, 1.0, 5.0],
      "color": [0.3, 0.5, 0.3, 1.0],
      "metallic": 0.0,
      "roughness": 0.5,
      "emissive": [0.0, 0.0, 0.0, 1.0],
      "light_color": [1.0, 1.0, 1.0, 1.0],
      "light_intensity": 800.0,
      "light_range": 20.0,
      "light_shadows": true,
      "spot_angle": 45.0,
      "visible": true
    },
    {
      "id": 2,
      "name": "Sun",
      "kind": {"Light": "Directional"},
      "children": [],
      "translation": [0.0, 0.0, 0.0],
      "rotation_euler": [-45.0, 30.0, 0.0],
      "scale": [1.0, 1.0, 1.0],
      "color": [0.8, 0.8, 0.8, 1.0],
      "metallic": 0.0,
      "roughness": 0.5,
      "emissive": [0.0, 0.0, 0.0, 1.0],
      "light_color": [1.0, 1.0, 1.0, 1.0],
      "light_intensity": 2000.0,
      "light_range": 20.0,
      "light_shadows": true,
      "spot_angle": 45.0,
      "visible": true
    }
  ]
}"#;
        std::fs::write(&scene_path, default_scene).map_err(|e| e.to_string())?;
    }

    // Create an assets folder
    let assets_path = path.join("assets");
    if !assets_path.exists() {
        std::fs::create_dir_all(&assets_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
