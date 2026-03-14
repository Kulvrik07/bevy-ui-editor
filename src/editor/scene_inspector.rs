use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Frame, Margin, RichText};
use bevy_egui::EguiContexts;

use crate::model::{
    SceneChanged, SceneDocument, SceneLightKind, SceneNodeKind, ScenePrimitive, SceneSelection,
};

use super::launcher::{AppMode, AppModeRes};

pub fn scene_inspector_system(
    mut ctx: EguiContexts,
    mut doc: ResMut<SceneDocument>,
    selection: Res<SceneSelection>,
    mut changed: ResMut<SceneChanged>,
    app_mode: Res<AppModeRes>,
    editor: Res<crate::model::EditorState>,
) {
    if app_mode.mode != AppMode::Editor { return; }
    if editor.play_mode { return; }
    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    egui::SidePanel::right("inspector_panel")
        .default_width(300.0)
        .min_width(250.0)
        .max_width(450.0)
        .frame(Frame {
            fill: Color32::from_rgb(30, 30, 30),
            inner_margin: Margin::same(0),
            ..Default::default()
        })
        .show(egui_ctx, |ui| {
            // Header
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Inspector").strong().color(Color32::from_rgb(200, 200, 200)));
            });
            ui.add_space(2.0);
            ui.separator();

            let selected_id = match selection.selected {
                Some(id) => id,
                None => {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("No node selected").color(Color32::from_rgb(120, 120, 120)).italics());
                    });
                    return;
                }
            };

            let node = match doc.find_node_mut(selected_id) {
                Some(n) => n,
                None => {
                    ui.label("Node not found");
                    return;
                }
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);

                // ── Node Info ──────────────────────────────────────────
                section_header(ui, "📋", "Node Info");
                ui.add_space(2.0);
                ui.indent("node_info_indent", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Name").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                        ui.add_space(8.0);
                        if ui.text_edit_singleline(&mut node.name).changed() {
                            changed.dirty = true;
                        }
                    });
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Kind").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                        ui.add_space(14.0);
                        ui.label(
                            RichText::new(node.kind.to_string())
                                .color(Color32::from_rgb(130, 190, 255))
                                .size(12.0),
                        );
                    });
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Visible").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                        if ui.checkbox(&mut node.visible, "").changed() {
                            changed.dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ID").color(Color32::from_rgb(100, 100, 100)).size(10.0));
                        ui.label(RichText::new(format!("{}", node.id)).color(Color32::from_rgb(100, 100, 100)).size(10.0));
                    });
                });
                ui.add_space(4.0);

                // ── Transform ─────────────────────────────────────────
                section_header(ui, "🔄", "Transform");
                ui.add_space(2.0);
                ui.indent("transform_indent", |ui| {
                    let mut t_changed = false;
                    t_changed |= vec3_editor(ui, "Position", &mut node.translation, 0.05);
                    t_changed |= vec3_editor(ui, "Rotation", &mut node.rotation_euler, 0.5);
                    t_changed |= vec3_editor(ui, "Scale   ", &mut node.scale, 0.01);

                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        if ui.small_button("Reset Position").clicked() {
                            node.translation = [0.0, 0.0, 0.0];
                            t_changed = true;
                        }
                        if ui.small_button("Reset Rotation").clicked() {
                            node.rotation_euler = [0.0, 0.0, 0.0];
                            t_changed = true;
                        }
                        if ui.small_button("Reset Scale").clicked() {
                            node.scale = [1.0, 1.0, 1.0];
                            t_changed = true;
                        }
                    });

                    if t_changed {
                        changed.dirty = true;
                    }
                });
                ui.add_space(4.0);

                // ── Material (meshes only) ────────────────────────────
                if matches!(node.kind, SceneNodeKind::Mesh(_)) {
                    section_header(ui, "🎨", "Material");
                    ui.add_space(2.0);
                    ui.indent("material_indent", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Color").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(16.0);
                            if ui.color_edit_button_rgba_premultiplied(&mut node.color).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Metallic").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(4.0);
                            if ui.add(egui::Slider::new(&mut node.metallic, 0.0..=1.0).step_by(0.01)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Roughness").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.add(egui::Slider::new(&mut node.roughness, 0.0..=1.0).step_by(0.01)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Emissive").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(2.0);
                            if ui.color_edit_button_rgba_premultiplied(&mut node.emissive).changed() {
                                changed.dirty = true;
                            }
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Mesh Type").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if let SceneNodeKind::Mesh(ref mut prim) = node.kind {
                                let prims = [
                                    ScenePrimitive::Cube,
                                    ScenePrimitive::Sphere,
                                    ScenePrimitive::Cylinder,
                                    ScenePrimitive::Capsule,
                                    ScenePrimitive::Plane,
                                    ScenePrimitive::Torus,
                                ];
                                egui::ComboBox::from_id_salt("mesh_prim")
                                    .selected_text(prim.to_string())
                                    .show_ui(ui, |ui| {
                                        for p in &prims {
                                            if ui.selectable_value(prim, *p, p.to_string()).changed() {
                                                changed.dirty = true;
                                            }
                                        }
                                    });
                            }
                        });
                    });
                    ui.add_space(4.0);
                }

                // ── Light ─────────────────────────────────────────────
                if matches!(node.kind, SceneNodeKind::Light(_)) {
                    section_header(ui, "💡", "Light");
                    ui.add_space(2.0);
                    ui.indent("light_indent", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Color").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(18.0);
                            if ui.color_edit_button_rgba_premultiplied(&mut node.light_color).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Intensity").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(2.0);
                            if ui.add(egui::DragValue::new(&mut node.light_intensity).speed(10.0).range(0.0..=100000.0)).changed() {
                                changed.dirty = true;
                            }
                        });

                        let is_point_or_spot = matches!(
                            node.kind,
                            SceneNodeKind::Light(SceneLightKind::Point) | SceneNodeKind::Light(SceneLightKind::Spot)
                        );

                        if is_point_or_spot {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Range").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                ui.add_space(18.0);
                                if ui.add(egui::DragValue::new(&mut node.light_range).speed(0.5).range(0.1..=200.0)).changed() {
                                    changed.dirty = true;
                                }
                            });
                        }

                        if matches!(node.kind, SceneNodeKind::Light(SceneLightKind::Spot)) {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Spot Angle").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                if ui.add(egui::Slider::new(&mut node.spot_angle, 1.0..=90.0).suffix("°")).changed() {
                                    changed.dirty = true;
                                }
                            });
                        }

                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Shadows").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(6.0);
                            if ui.checkbox(&mut node.light_shadows, "").changed() {
                                changed.dirty = true;
                            }
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Type").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(22.0);
                            if let SceneNodeKind::Light(ref mut lk) = node.kind {
                                let kinds = [
                                    SceneLightKind::Point,
                                    SceneLightKind::Directional,
                                    SceneLightKind::Spot,
                                ];
                                egui::ComboBox::from_id_salt("light_kind")
                                    .selected_text(lk.to_string())
                                    .show_ui(ui, |ui| {
                                        for k in &kinds {
                                            if ui.selectable_value(lk, *k, k.to_string()).changed() {
                                                changed.dirty = true;
                                            }
                                        }
                                    });
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            });
        });
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, icon: &str, title: &str) {
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label(RichText::new(icon).size(13.0));
        ui.label(RichText::new(title).strong().color(Color32::from_rgb(180, 200, 230)).size(13.0));
    });
    let rect = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 6.0, rect.top()),
            egui::pos2(rect.right() - 6.0, rect.top()),
        ],
        egui::Stroke::new(0.5, Color32::from_rgb(60, 60, 70)),
    );
}

fn vec3_editor(ui: &mut egui::Ui, label: &str, vals: &mut [f32; 3], speed: f64) -> bool {
    let mut any_changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(Color32::from_rgb(140, 140, 140)).size(11.0).monospace());
        ui.add_space(4.0);
        let labels = ["X", "Y", "Z"];
        let colors = [
            Color32::from_rgb(220, 80, 80),
            Color32::from_rgb(80, 200, 80),
            Color32::from_rgb(80, 120, 220),
        ];
        for i in 0..3 {
            ui.colored_label(colors[i], RichText::new(labels[i]).size(11.0));
            if ui.add(egui::DragValue::new(&mut vals[i]).speed(speed).max_decimals(2)).changed() {
                any_changed = true;
            }
        }
    });
    any_changed
}
