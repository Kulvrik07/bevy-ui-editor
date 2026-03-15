use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Frame, Margin, RichText};
use bevy_egui::EguiContexts;

use crate::model::{
    SceneChanged, SceneDocument, SceneLightKind, SceneNodeKind, ScenePrimitive, SceneSelection,
    ScriptRef, EditorState, SceneAlphaMode, SceneProjection, PhysicsBodyType, ColliderShape,
    EnvironmentSettings, ToneMapping,
};

use super::file_explorer::FileExplorerState;
use super::launcher::{AppMode, AppModeRes};

pub fn scene_inspector_system(
    mut ctx: EguiContexts,
    mut doc: ResMut<SceneDocument>,
    selection: Res<SceneSelection>,
    mut changed: ResMut<SceneChanged>,
    app_mode: Res<AppModeRes>,
    editor: Res<EditorState>,
    file_explorer: Res<FileExplorerState>,
    mut env: ResMut<EnvironmentSettings>,
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
                    // Show environment settings when nothing selected
                    environment_settings_ui(ui, &mut env);
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

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Alpha Mode").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            let modes = [SceneAlphaMode::Opaque, SceneAlphaMode::Blend, SceneAlphaMode::Mask, SceneAlphaMode::AlphaToCoverage];
                            egui::ComboBox::from_id_salt("alpha_mode")
                                .selected_text(node.alpha_mode.to_string())
                                .show_ui(ui, |ui| {
                                    for m in &modes {
                                        if ui.selectable_value(&mut node.alpha_mode, *m, m.to_string()).changed() {
                                            changed.dirty = true;
                                        }
                                    }
                                });
                        });
                        if node.alpha_mode == SceneAlphaMode::Mask {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Alpha Cutoff").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                if ui.add(egui::Slider::new(&mut node.alpha_cutoff, 0.0..=1.0).step_by(0.01)).changed() {
                                    changed.dirty = true;
                                }
                            });
                        }

                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Unlit").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(22.0);
                            if ui.checkbox(&mut node.unlit, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Double Sided").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.checkbox(&mut node.double_sided, "").changed() { changed.dirty = true; }
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Texture").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(8.0);
                            if ui.add(egui::TextEdit::singleline(&mut node.texture_path).hint_text("path/to/texture.png").desired_width(150.0)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Normal Map").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.add(egui::TextEdit::singleline(&mut node.normal_map_path).hint_text("path/to/normal.png").desired_width(150.0)).changed() {
                                changed.dirty = true;
                            }
                        });
                    });
                    ui.add_space(4.0);

                    // ── Render Settings ───────────────────────────────
                    section_header(ui, "🔲", "Render");
                    ui.add_space(2.0);
                    ui.indent("render_indent", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Cast Shadows").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.checkbox(&mut node.cast_shadows, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Receive Shadows").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.checkbox(&mut node.receive_shadows, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Render Layer").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(2.0);
                            let mut layer = node.render_layer as i32;
                            if ui.add(egui::DragValue::new(&mut layer).range(0..=31)).changed() {
                                node.render_layer = layer.clamp(0, 31) as u8;
                                changed.dirty = true;
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

                // ── Model info ────────────────────────────────────────
                if let SceneNodeKind::Model(ref path) = node.kind {
                    section_header(ui, "🎨", "3D Model");
                    ui.add_space(2.0);
                    ui.indent("model_indent", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Asset").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(path.as_str())
                                    .color(Color32::from_rgb(130, 200, 190))
                                    .size(11.0),
                            );
                        });
                        ui.add_space(2.0);
                        let file_name = std::path::Path::new(path.as_str())
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let ext = file_name.rsplit('.').next().unwrap_or("").to_lowercase();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Format").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(4.0);
                            let format_text = match ext.as_str() {
                                "glb" => "glTF Binary (.glb)",
                                "gltf" => "glTF (.gltf)",
                                "fbx" => "FBX (requires converter)",
                                "obj" => "OBJ (requires converter)",
                                _ => "Unknown format",
                            };
                            ui.label(
                                RichText::new(format_text)
                                    .color(Color32::from_rgb(160, 160, 170))
                                    .size(11.0),
                            );
                        });
                    });
                    ui.add_space(4.0);
                }

                // ── Camera Properties ─────────────────────────────────
                if matches!(node.kind, SceneNodeKind::Camera) {
                    section_header(ui, "📷", "Camera");
                    ui.add_space(2.0);
                    ui.indent("camera_indent", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Projection").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            let projs = [SceneProjection::Perspective, SceneProjection::Orthographic];
                            egui::ComboBox::from_id_salt("cam_proj")
                                .selected_text(node.projection.to_string())
                                .show_ui(ui, |ui| {
                                    for p in &projs {
                                        if ui.selectable_value(&mut node.projection, *p, p.to_string()).changed() {
                                            changed.dirty = true;
                                        }
                                    }
                                });
                        });
                        if node.projection == SceneProjection::Perspective {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("FOV").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                ui.add_space(32.0);
                                if ui.add(egui::Slider::new(&mut node.fov, 10.0..=120.0).suffix("°")).changed() {
                                    changed.dirty = true;
                                }
                            });
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Near Clip").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(6.0);
                            if ui.add(egui::DragValue::new(&mut node.near_clip).speed(0.01).range(0.001..=100.0)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Far Clip").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(12.0);
                            if ui.add(egui::DragValue::new(&mut node.far_clip).speed(10.0).range(10.0..=100000.0)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("HDR").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(28.0);
                            if ui.checkbox(&mut node.hdr, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Active Camera").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            if ui.checkbox(&mut node.is_active_camera, "").changed() { changed.dirty = true; }
                        });
                    });
                    ui.add_space(4.0);
                }

                // ── Audio Properties ──────────────────────────────────
                if matches!(node.kind, SceneNodeKind::AudioSource(_)) {
                    section_header(ui, "🔊", "Audio");
                    ui.add_space(2.0);
                    ui.indent("audio_indent", |ui| {
                        // Clone the path to avoid borrow conflict with node.kind
                        let mut audio_path = if let SceneNodeKind::AudioSource(ref p) = node.kind { p.clone() } else { String::new() };
                        let mut path_changed = false;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Source").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(12.0);
                            if ui.add(egui::TextEdit::singleline(&mut audio_path).hint_text("audio/sound.ogg").desired_width(150.0)).changed() {
                                path_changed = true;
                            }
                        });
                        if path_changed {
                            if let SceneNodeKind::AudioSource(ref mut p) = node.kind { *p = audio_path; }
                            changed.dirty = true;
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Volume").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(10.0);
                            if ui.add(egui::Slider::new(&mut node.audio_volume, 0.0..=2.0).step_by(0.01)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Looping").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(10.0);
                            if ui.checkbox(&mut node.audio_looping, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Spatial").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(14.0);
                            if ui.checkbox(&mut node.audio_spatial, "").changed() { changed.dirty = true; }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Autoplay").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(6.0);
                            if ui.checkbox(&mut node.audio_autoplay, "").changed() { changed.dirty = true; }
                        });
                    });
                    ui.add_space(4.0);
                }

                // ── Physics ───────────────────────────────────────────
                section_header(ui, "⚙", "Physics");
                ui.add_space(2.0);
                ui.indent("physics_indent", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Body Type").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                        ui.add_space(4.0);
                        let types = [PhysicsBodyType::None, PhysicsBodyType::Static, PhysicsBodyType::Dynamic, PhysicsBodyType::Kinematic];
                        egui::ComboBox::from_id_salt("phys_body")
                            .selected_text(node.physics_body.to_string())
                            .show_ui(ui, |ui| {
                                for t in &types {
                                    if ui.selectable_value(&mut node.physics_body, *t, t.to_string()).changed() {
                                        changed.dirty = true;
                                    }
                                }
                            });
                    });

                    if node.physics_body != PhysicsBodyType::None {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Collider").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(12.0);
                            let shapes = [ColliderShape::None, ColliderShape::Box, ColliderShape::Sphere, ColliderShape::Capsule, ColliderShape::Cylinder, ColliderShape::Auto];
                            egui::ComboBox::from_id_salt("phys_collider")
                                .selected_text(node.collider_shape.to_string())
                                .show_ui(ui, |ui| {
                                    for s in &shapes {
                                        if ui.selectable_value(&mut node.collider_shape, *s, s.to_string()).changed() {
                                            changed.dirty = true;
                                        }
                                    }
                                });
                        });
                        if node.physics_body == PhysicsBodyType::Dynamic {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Mass").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                ui.add_space(24.0);
                                if ui.add(egui::DragValue::new(&mut node.mass).speed(0.1).range(0.01..=10000.0)).changed() {
                                    changed.dirty = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Gravity Scale").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                if ui.add(egui::DragValue::new(&mut node.gravity_scale).speed(0.05).range(-10.0..=10.0)).changed() {
                                    changed.dirty = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Lock Rotation").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                                if ui.checkbox(&mut node.lock_rotation, "").changed() { changed.dirty = true; }
                            });
                        }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Friction").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(16.0);
                            if ui.add(egui::Slider::new(&mut node.friction, 0.0..=2.0).step_by(0.01)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Restitution").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                            ui.add_space(2.0);
                            if ui.add(egui::Slider::new(&mut node.restitution, 0.0..=1.0).step_by(0.01)).changed() {
                                changed.dirty = true;
                            }
                        });
                    }
                });
                ui.add_space(4.0);

                // ── Scripts ───────────────────────────────────────────
                section_header(ui, "📜", "Scripts");
                ui.add_space(2.0);
                ui.indent("scripts_indent", |ui| {
                    let mut remove_idx: Option<usize> = None;
                    let mut toggle_idx: Option<usize> = None;
                    for (i, script) in node.scripts.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let icon_color = if script.enabled {
                                Color32::from_rgb(100, 200, 100)
                            } else {
                                Color32::from_rgb(120, 120, 120)
                            };
                            if ui.add(egui::Button::new(
                                RichText::new(if script.enabled { "●" } else { "○" })
                                    .color(icon_color)
                                    .size(11.0),
                            ).fill(Color32::TRANSPARENT).frame(false)).on_hover_text(
                                if script.enabled { "Enabled (click to disable)" } else { "Disabled (click to enable)" }
                            ).clicked() {
                                toggle_idx = Some(i);
                            }
                            let display_name = std::path::Path::new(&script.path)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| script.path.clone());
                            ui.label(
                                RichText::new(&display_name)
                                    .color(if script.enabled {
                                        Color32::from_rgb(200, 200, 205)
                                    } else {
                                        Color32::from_rgb(120, 120, 130)
                                    })
                                    .size(11.0),
                            ).on_hover_text(&script.path);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").on_hover_text("Remove script").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        });
                    }
                    if let Some(idx) = toggle_idx {
                        node.scripts[idx].enabled = !node.scripts[idx].enabled;
                        changed.dirty = true;
                    }
                    if let Some(idx) = remove_idx {
                        node.scripts.remove(idx);
                        changed.dirty = true;
                    }
                    ui.add_space(4.0);

                    // Scan project for available script files
                    let project_root = &file_explorer.root;
                    let available_scripts = scan_script_files(project_root);
                    let attached_paths: Vec<String> = node.scripts.iter().map(|s| s.path.clone()).collect();

                    if !available_scripts.is_empty() {
                        ui.menu_button(
                            RichText::new("+ Add Script").color(Color32::from_rgb(130, 190, 255)).size(11.0),
                            |ui| {
                                ui.set_min_width(200.0);
                                if available_scripts.iter().all(|s| attached_paths.contains(s)) {
                                    ui.label(RichText::new("All scripts already attached").color(Color32::from_rgb(120, 120, 130)).italics().size(11.0));
                                }
                                for script_path in &available_scripts {
                                    if attached_paths.contains(script_path) {
                                        continue;
                                    }
                                    let display = std::path::Path::new(script_path)
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| script_path.clone());
                                    if ui.button(format!("📜 {display}")).on_hover_text(script_path).clicked() {
                                        node.scripts.push(ScriptRef {
                                            path: script_path.clone(),
                                            enabled: true,
                                        });
                                        changed.dirty = true;
                                        ui.close();
                                    }
                                }
                            },
                        );
                    } else {
                        ui.label(
                            RichText::new("No scripts in project.\nCreate scripts via File Explorer\n(right-click → New Script)")
                                .color(Color32::from_rgb(100, 100, 110))
                                .italics()
                                .size(10.0),
                        );
                    }

                    if node.scripts.is_empty() && !available_scripts.is_empty() {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("No scripts attached")
                                .color(Color32::from_rgb(100, 100, 110))
                                .italics()
                                .size(11.0),
                        );
                    }
                });
                ui.add_space(4.0);
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

/// Scan the project for script files (.rs, .lua, .rhai, .wasm) relative to project root
fn scan_script_files(project_root: &std::path::Path) -> Vec<String> {
    let mut scripts = Vec::new();
    let scripts_dir = project_root.join("scripts");
    if scripts_dir.is_dir() {
        collect_scripts_recursive(&scripts_dir, project_root, &mut scripts);
    }
    // Also check src/ for .rs files
    let src_dir = project_root.join("src");
    if src_dir.is_dir() {
        collect_scripts_recursive(&src_dir, project_root, &mut scripts);
    }
    scripts.sort();
    scripts
}

fn collect_scripts_recursive(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" {
                collect_scripts_recursive(&path, root, out);
            }
        } else {
            let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
            if matches!(ext.as_str(), "rs" | "lua" | "rhai" | "wasm") {
                if let Ok(rel) = path.strip_prefix(root) {
                    out.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }
}

// ─── Environment settings (shown when no node is selected) ────────────────────

fn environment_settings_ui(ui: &mut egui::Ui, env: &mut EnvironmentSettings) {
    ui.add_space(8.0);
    section_header(ui, "🌍", "Environment");
    ui.add_space(4.0);
    ui.indent("env_indent", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Clear Color").color(Color32::from_rgb(160, 160, 160)).size(12.0));
            ui.color_edit_button_rgba_premultiplied(&mut env.clear_color);
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("Ambient Color").color(Color32::from_rgb(160, 160, 160)).size(12.0));
            ui.color_edit_button_rgba_premultiplied(&mut env.ambient_color);
        });
        ui.horizontal(|ui| {
            ui.label(RichText::new("Ambient Brightness").color(Color32::from_rgb(160, 160, 160)).size(12.0));
            ui.add(egui::Slider::new(&mut env.ambient_brightness, 0.0..=5.0).step_by(0.01));
        });
    });
    ui.add_space(4.0);

    section_header(ui, "🌫", "Fog");
    ui.add_space(4.0);
    ui.indent("fog_indent", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Enabled").color(Color32::from_rgb(160, 160, 160)).size(12.0));
            ui.add_space(10.0);
            ui.checkbox(&mut env.fog_enabled, "");
        });
        if env.fog_enabled {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Color").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add_space(20.0);
                ui.color_edit_button_rgba_premultiplied(&mut env.fog_color);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Start").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add_space(22.0);
                ui.add(egui::DragValue::new(&mut env.fog_start).speed(0.5).range(0.0..=1000.0));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("End").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add_space(28.0);
                ui.add(egui::DragValue::new(&mut env.fog_end).speed(0.5).range(0.0..=5000.0));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Density").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add_space(10.0);
                ui.add(egui::Slider::new(&mut env.fog_density, 0.0..=1.0).step_by(0.001));
            });
        }
    });
    ui.add_space(4.0);

    section_header(ui, "✨", "Bloom");
    ui.add_space(4.0);
    ui.indent("bloom_indent", |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Enabled").color(Color32::from_rgb(160, 160, 160)).size(12.0));
            ui.add_space(10.0);
            ui.checkbox(&mut env.bloom_enabled, "");
        });
        if env.bloom_enabled {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Intensity").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add_space(4.0);
                ui.add(egui::Slider::new(&mut env.bloom_intensity, 0.0..=2.0).step_by(0.01));
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Threshold").color(Color32::from_rgb(160, 160, 160)).size(12.0));
                ui.add(egui::Slider::new(&mut env.bloom_threshold, 0.0..=5.0).step_by(0.01));
            });
        }
    });
    ui.add_space(4.0);

    section_header(ui, "🎬", "Tone Mapping");
    ui.add_space(4.0);
    ui.indent("tone_indent", |ui| {
        let mappings = [
            ToneMapping::None, ToneMapping::Reinhard, ToneMapping::ReinhardLuminance,
            ToneMapping::AcesFitted, ToneMapping::AgX, ToneMapping::SomewhatBoringDisplayTransform,
            ToneMapping::TonyMcMapface, ToneMapping::BlenderFilmic,
        ];
        egui::ComboBox::from_id_salt("tone_mapping")
            .selected_text(env.tone_mapping.to_string())
            .show_ui(ui, |ui| {
                for tm in &mappings {
                    ui.selectable_value(&mut env.tone_mapping, *tm, tm.to_string());
                }
            });
    });
    ui.add_space(8.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Select a node to inspect its properties").color(Color32::from_rgb(100, 100, 110)).italics().size(10.0));
    });
}
