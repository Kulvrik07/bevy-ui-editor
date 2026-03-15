use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Frame, Margin, RichText};
use bevy_egui::EguiContexts;

use crate::model::{
    new_scene_node, ConsoleLog, EditorState, SceneChanged, SceneDocument, SceneIdCounter,
    SceneLightKind, SceneNodeKind, ScenePrimitive, SceneSelection, TransformMode, UndoHistory,
};

use super::file_explorer::FileExplorerState;
use super::launcher::{AppMode, AppModeRes};
use super::viewport3d::OrbitState;

pub fn scene_toolbar_system(
    mut ctx: EguiContexts,
    mut doc: ResMut<SceneDocument>,
    mut selection: ResMut<SceneSelection>,
    mut id_counter: ResMut<SceneIdCounter>,
    mut changed: ResMut<SceneChanged>,
    mut undo: ResMut<UndoHistory>,
    mut editor: ResMut<EditorState>,
    mut console: ResMut<ConsoleLog>,
    mut orbit: ResMut<OrbitState>,
    app_mode: Res<AppModeRes>,
    file_explorer: Res<FileExplorerState>,
) {
    if app_mode.mode != AppMode::Editor { return; }
    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    // In play mode, show minimal toolbar with Stop button
    if editor.play_mode {
        egui::TopBottomPanel::top("toolbar_play")
            .frame(Frame {
                fill: Color32::from_rgb(50, 30, 30),
                inner_margin: Margin::symmetric(8, 4),
                ..Default::default()
            })
            .show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("▶ PLAYING").color(Color32::from_rgb(80, 220, 80)).strong().size(14.0));
                    ui.separator();
                    ui.label(RichText::new("WASD: Move | Space/Ctrl: Up/Down | RMB+Mouse: Look | Shift: Sprint | Esc: Stop").size(11.0).color(Color32::from_rgb(180, 180, 180)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let stop_btn = egui::Button::new(
                            RichText::new("⏹ Stop").color(Color32::from_rgb(220, 80, 80)).size(14.0).strong(),
                        );
                        if ui.add(stop_btn).clicked() {
                            // Restore orbit state
                            if let Some((yaw, pitch, distance, focus)) = editor.saved_orbit.take() {
                                orbit.yaw = yaw;
                                orbit.pitch = pitch;
                                orbit.distance = distance;
                                orbit.focus = Vec3::from_array(focus);
                            }
                            editor.play_mode = false;
                            console.info("Exited play mode");
                        }
                    });
                });
            });
        return;
    }

    egui::TopBottomPanel::top("toolbar")
        .frame(Frame {
            fill: Color32::from_rgb(38, 38, 38),
            inner_margin: Margin::symmetric(8, 4),
            ..Default::default()
        })
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                // ── Title ─────────────────────────────────────────────
                ui.label(RichText::new("🎮 Bevy 3D Editor").strong().color(Color32::from_rgb(120, 180, 255)).size(15.0));
                ui.separator();

                // ── File menu ─────────────────────────────────────────
                ui.menu_button(RichText::new("File").size(13.0), |ui| {
                    if ui.button("💾 Save Scene").clicked() {
                        save_scene(&doc, &mut editor, &mut console);
                        ui.close();
                    }
                    if ui.button("💾 Save As...").clicked() {
                        save_scene_as(&doc, &mut editor, &mut console);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("📦 Export Project").clicked() {
                        export_project_action(&doc, &file_explorer, &mut console);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("📂 New Scene").clicked() {
                        undo.push_snapshot(&doc.nodes);
                        *doc = SceneDocument::default();
                        editor.scene_file_path = None;
                        editor.scene_dirty = false;
                        changed.dirty = true;
                        console.info("New scene created");
                        ui.close();
                    }
                });

                ui.separator();

                // ── Add objects ────────────────────────────────────────
                ui.menu_button(RichText::new("+ Mesh").size(13.0), |ui| {
                    let prims = [
                        ("Cube", ScenePrimitive::Cube),
                        ("Sphere", ScenePrimitive::Sphere),
                        ("Cylinder", ScenePrimitive::Cylinder),
                        ("Capsule", ScenePrimitive::Capsule),
                        ("Plane", ScenePrimitive::Plane),
                        ("Torus", ScenePrimitive::Torus),
                    ];
                    for (label, prim) in prims {
                        if ui.button(label).clicked() {
                            let id = id_counter.next_id();
                            let node = new_scene_node(id, SceneNodeKind::Mesh(prim));
                            undo.push_snapshot(&doc.nodes);
                            doc.add_node(selection.selected, node);
                            selection.selected = Some(id);
                            changed.dirty = true;
                            editor.scene_dirty = true;
                            console.info(format!("Added {label}"));
                            ui.close();
                        }
                    }
                });

                ui.menu_button(RichText::new("+ Light").size(13.0), |ui| {
                    let lights = [
                        ("Point Light", SceneLightKind::Point),
                        ("Directional Light", SceneLightKind::Directional),
                        ("Spot Light", SceneLightKind::Spot),
                    ];
                    for (label, lk) in lights {
                        if ui.button(label).clicked() {
                            let id = id_counter.next_id();
                            let node = new_scene_node(id, SceneNodeKind::Light(lk));
                            undo.push_snapshot(&doc.nodes);
                            doc.add_node(selection.selected, node);
                            selection.selected = Some(id);
                            changed.dirty = true;
                            editor.scene_dirty = true;
                            console.info(format!("Added {label}"));
                            ui.close();
                        }
                    }
                });

                if ui.button(RichText::new("+ Empty").size(13.0)).clicked() {
                    let id = id_counter.next_id();
                    let node = new_scene_node(id, SceneNodeKind::Empty);
                    undo.push_snapshot(&doc.nodes);
                    doc.add_node(selection.selected, node);
                    selection.selected = Some(id);
                    changed.dirty = true;
                    editor.scene_dirty = true;
                    console.info("Added Empty node");
                }

                if ui.button(RichText::new("+ Camera").size(13.0)).clicked() {
                    let id = id_counter.next_id();
                    let node = new_scene_node(id, SceneNodeKind::Camera);
                    undo.push_snapshot(&doc.nodes);
                    doc.add_node(selection.selected, node);
                    selection.selected = Some(id);
                    changed.dirty = true;
                    editor.scene_dirty = true;
                    console.info("Added Camera");
                }

                if ui.button(RichText::new("+ Audio").size(13.0)).clicked() {
                    let id = id_counter.next_id();
                    let node = new_scene_node(id, SceneNodeKind::AudioSource(String::new()));
                    undo.push_snapshot(&doc.nodes);
                    doc.add_node(selection.selected, node);
                    selection.selected = Some(id);
                    changed.dirty = true;
                    editor.scene_dirty = true;
                    console.info("Added AudioSource");
                }

                ui.separator();

                // ── Undo / Redo ───────────────────────────────────────
                if ui
                    .add_enabled(undo.can_undo(), egui::Button::new(RichText::new("↩").size(16.0)))
                    .on_hover_text("Undo (Ctrl+Z)")
                    .clicked()
                {
                    if let Some(prev) = undo.undo(&doc.nodes) {
                        doc.nodes = prev;
                        changed.dirty = true;
                        console.info("Undo");
                    }
                }
                if ui
                    .add_enabled(undo.can_redo(), egui::Button::new(RichText::new("↪").size(16.0)))
                    .on_hover_text("Redo (Ctrl+Y)")
                    .clicked()
                {
                    if let Some(next) = undo.redo(&doc.nodes) {
                        doc.nodes = next;
                        changed.dirty = true;
                        console.info("Redo");
                    }
                }

                ui.separator();

                // ── Delete / Duplicate ────────────────────────────────
                if ui
                    .add_enabled(selection.selected.is_some(), egui::Button::new(RichText::new("🗑").size(14.0)))
                    .on_hover_text("Delete (Del)")
                    .clicked()
                {
                    if let Some(sel) = selection.selected {
                        undo.push_snapshot(&doc.nodes);
                        doc.remove_node(sel);
                        selection.selected = None;
                        changed.dirty = true;
                        editor.scene_dirty = true;
                        console.info("Deleted node");
                    }
                }
                if ui
                    .add_enabled(selection.selected.is_some(), egui::Button::new(RichText::new("📋").size(14.0)))
                    .on_hover_text("Duplicate (Ctrl+D)")
                    .clicked()
                {
                    if let Some(sel) = selection.selected {
                        if let Some(node) = doc.find_node(sel) {
                            let mut dup = node.clone();
                            let new_id = id_counter.next_id();
                            dup.id = new_id;
                            dup.name = format!("{} (copy)", dup.name);
                            dup.translation[0] += 1.0;
                            undo.push_snapshot(&doc.nodes);
                            doc.add_node(None, dup);
                            selection.selected = Some(new_id);
                            changed.dirty = true;
                            editor.scene_dirty = true;
                            console.info("Duplicated node");
                        }
                    }
                }

                ui.separator();

                // ── Transform mode ────────────────────────────────────
                let modes = [
                    (TransformMode::Select, "🖱", "Select (Q)"),
                    (TransformMode::Translate, "✥", "Move (W)"),
                    (TransformMode::Rotate, "🔄", "Rotate (E)"),
                    (TransformMode::Scale, "⇔", "Scale (R)"),
                ];
                for (mode, icon, tooltip) in modes {
                    let is_active = editor.transform_mode == mode;
                    let btn = egui::Button::new(
                        RichText::new(icon).size(15.0).color(if is_active {
                            Color32::from_rgb(100, 180, 255)
                        } else {
                            Color32::from_rgb(180, 180, 180)
                        }),
                    );
                    if ui.add(btn).on_hover_text(tooltip).clicked() {
                        editor.transform_mode = mode;
                    }
                }

                ui.separator();

                // ── View toggles ──────────────────────────────────────
                let grid_text = if editor.show_grid { "Grid ✓" } else { "Grid" };
                if ui.button(RichText::new(grid_text).size(12.0)).clicked() {
                    editor.show_grid = !editor.show_grid;
                }

                let stats_text = if editor.show_stats { "Stats ✓" } else { "Stats" };
                if ui.button(RichText::new(stats_text).size(12.0)).clicked() {
                    editor.show_stats = !editor.show_stats;
                }

                let snap_text = if editor.snap_enabled { "Snap ✓" } else { "Snap" };
                if ui.button(RichText::new(snap_text).size(12.0)).on_hover_text(
                    format!("Snap to grid (T:{} R:{}° S:{})", editor.snap_translate, editor.snap_rotate, editor.snap_scale)
                ).clicked() {
                    editor.snap_enabled = !editor.snap_enabled;
                }

                // ── Spacer ────────────────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // ── Play button ───────────────────────────────────
                    let play_btn = egui::Button::new(
                        RichText::new("▶ Play").color(Color32::from_rgb(80, 220, 80)).size(14.0).strong(),
                    );
                    if ui.add(play_btn).on_hover_text("Enter play mode (WASD+Mouse)").clicked() {
                        // Save scene first
                        save_scene(&doc, &mut editor, &mut console);
                        // Save orbit state and enter play mode
                        editor.saved_orbit = Some((orbit.yaw, orbit.pitch, orbit.distance, orbit.focus.to_array()));
                        editor.play_mode = true;
                        changed.dirty = true; // Ensure scene re-syncs for physics
                        console.info("Entered play mode — WASD to move, RMB+Mouse to look, Esc to stop");
                    }

                    ui.separator();

                    // Scene name
                    let scene_name = editor
                        .scene_file_path
                        .as_ref()
                        .and_then(|p| p.rsplit('/').next().map(|s| s.to_string()))
                        .unwrap_or_else(|| "Unsaved Scene".to_string());
                    let dirty_marker = if editor.scene_dirty { " •" } else { "" };
                    ui.label(
                        RichText::new(format!("{scene_name}{dirty_marker}"))
                            .color(Color32::from_rgb(160, 160, 160))
                            .size(12.0),
                    );
                });
            });
        });
}

// ─── Save helpers ─────────────────────────────────────────────────────────────

fn save_scene(doc: &SceneDocument, editor: &mut EditorState, console: &mut ConsoleLog) {
    let path = editor
        .scene_file_path
        .clone()
        .unwrap_or_else(|| "scene.json".to_string());

    match doc.to_json() {
        Ok(json) => match std::fs::write(&path, json) {
            Ok(()) => {
                editor.scene_file_path = Some(path.clone());
                editor.scene_dirty = false;
                console.info(format!("Scene saved to {path}"));
            }
            Err(e) => console.error(format!("Save failed: {e}")),
        },
        Err(e) => console.error(format!("Serialization failed: {e}")),
    }
}

fn save_scene_as(doc: &SceneDocument, editor: &mut EditorState, console: &mut ConsoleLog) {
    let path = "scene.json".to_string();
    match doc.to_json() {
        Ok(json) => match std::fs::write(&path, json) {
            Ok(()) => {
                editor.scene_file_path = Some(path.clone());
                editor.scene_dirty = false;
                console.info(format!("Scene saved to {path}"));
            }
            Err(e) => console.error(format!("Save failed: {e}")),
        },
        Err(e) => console.error(format!("Serialization failed: {e}")),
    }
}

fn export_project_action(
    doc: &SceneDocument,
    file_explorer: &FileExplorerState,
    console: &mut ConsoleLog,
) {
    let project_root = &file_explorer.root;
    let output_dir = project_root.join("build");

    console.info(format!("Exporting project to {}", output_dir.display()));

    let result = crate::export::export_project(doc, project_root, &output_dir);

    if result.errors.is_empty() {
        console.info(format!(
            "✅ Project exported to {}\n  → cd {} && cargo run",
            result.output_dir.display(),
            result.output_dir.display()
        ));
    } else {
        for err in &result.errors {
            console.warn(format!("Export warning: {err}"));
        }
        console.info(format!(
            "⚠ Project exported with {} warning(s) to {}",
            result.errors.len(),
            result.output_dir.display()
        ));
    }
}
