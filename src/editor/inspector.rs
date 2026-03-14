use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::model::{
    EditorAlignItems, EditorChanged, EditorDocument, EditorFlexDirection, EditorJustifyContent,
    EditorNodeType, EditorOverflow, EditorPositionType, EditorSelection, EditorVal,
};

pub fn inspector_system(
    mut contexts: EguiContexts,
    mut document: ResMut<EditorDocument>,
    selection: Res<EditorSelection>,
    mut changed: ResMut<EditorChanged>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::SidePanel::right("inspector_panel")
        .exact_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();

            let selected_id = match selection.selected {
                Some(id) => id,
                None => {
                    ui.label("Select a node to inspect");
                    return;
                }
            };

            let node = match document.find_node_mut(selected_id) {
                Some(n) => n,
                None => {
                    ui.label("Node not found");
                    return;
                }
            };

            egui::ScrollArea::vertical().show(ui, |ui| {
                // ── Node Info ──────────────────────────────────────────────────
                egui::CollapsingHeader::new("Node Info")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            if ui.text_edit_singleline(&mut node.name).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Type:");
                            let types = [
                                EditorNodeType::Container,
                                EditorNodeType::Text,
                                EditorNodeType::Button,
                                EditorNodeType::Image,
                            ];
                            egui::ComboBox::from_id_salt("node_type")
                                .selected_text(node.node_type.to_string())
                                .show_ui(ui, |ui| {
                                    for t in &types {
                                        if ui
                                            .selectable_value(&mut node.node_type, *t, t.to_string())
                                            .changed()
                                        {
                                            changed.dirty = true;
                                        }
                                    }
                                });
                        });
                    });

                // ── Layout ────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Layout")
                    .default_open(true)
                    .show(ui, |ui| {
                        val_editor(ui, "Width", &mut node.width, &mut changed.dirty);
                        val_editor(ui, "Height", &mut node.height, &mut changed.dirty);

                        ui.horizontal(|ui| {
                            ui.label("Flex Dir:");
                            combo_flex_dir(ui, &mut node.flex_direction, &mut changed.dirty);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Justify:");
                            combo_justify(ui, &mut node.justify_content, &mut changed.dirty);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Align Items:");
                            combo_align(ui, &mut node.align_items, &mut changed.dirty);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Position:");
                            combo_position(ui, &mut node.position_type, &mut changed.dirty);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Flex Grow:");
                            if ui.add(egui::DragValue::new(&mut node.flex_grow).speed(0.1).range(0.0..=100.0)).changed() {
                                changed.dirty = true;
                            }
                            ui.label("Shrink:");
                            if ui.add(egui::DragValue::new(&mut node.flex_shrink).speed(0.1).range(0.0..=100.0)).changed() {
                                changed.dirty = true;
                            }
                        });

                        val_editor(ui, "Flex Basis", &mut node.flex_basis, &mut changed.dirty);
                        val_editor(ui, "Row Gap", &mut node.row_gap, &mut changed.dirty);
                        val_editor(ui, "Col Gap", &mut node.column_gap, &mut changed.dirty);

                        ui.horizontal(|ui| {
                            ui.label("Overflow X:");
                            combo_overflow(ui, "ox", &mut node.overflow_x, &mut changed.dirty);
                            ui.label("Y:");
                            combo_overflow(ui, "oy", &mut node.overflow_y, &mut changed.dirty);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Flex Wrap:");
                            if ui.checkbox(&mut node.flex_wrap, "").changed() {
                                changed.dirty = true;
                            }
                        });
                    });

                // ── Spacing ───────────────────────────────────────────────────
                egui::CollapsingHeader::new("Spacing")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label("Padding:");
                        rect_editor(ui, "pad", &mut node.padding, &mut changed.dirty);
                        ui.label("Margin:");
                        rect_editor(ui, "mar", &mut node.margin, &mut changed.dirty);
                        ui.label("Border:");
                        rect_editor(ui, "brd", &mut node.border, &mut changed.dirty);
                    });

                // ── Visual ────────────────────────────────────────────────────
                egui::CollapsingHeader::new("Visual")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Background:");
                            if ui.color_edit_button_rgba_premultiplied(&mut node.background_color).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Border Color:");
                            if ui.color_edit_button_rgba_premultiplied(&mut node.border_color).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Border Radius:");
                            if ui.add(egui::Slider::new(&mut node.border_radius, 0.0..=100.0)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Z-Index:");
                            if ui.add(egui::DragValue::new(&mut node.z_index)).changed() {
                                changed.dirty = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Visible:");
                            if ui.checkbox(&mut node.visible, "").changed() {
                                changed.dirty = true;
                            }
                        });
                    });

                // ── Text (only for Text / Button) ─────────────────────────────
                let show_text = matches!(node.node_type, EditorNodeType::Text | EditorNodeType::Button);
                if show_text {
                    egui::CollapsingHeader::new("Text")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("Content:");
                            if ui
                                .add(egui::TextEdit::multiline(&mut node.text_content).desired_rows(3))
                                .changed()
                            {
                                changed.dirty = true;
                            }
                            ui.horizontal(|ui| {
                                ui.label("Font Size:");
                                if ui.add(egui::Slider::new(&mut node.font_size, 8.0..=120.0)).changed() {
                                    changed.dirty = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("Text Color:");
                                if ui.color_edit_button_rgba_premultiplied(&mut node.text_color).changed() {
                                    changed.dirty = true;
                                }
                            });
                        });
                }
            });
        });
}

// ─── Helper widgets ───────────────────────────────────────────────────────────

fn val_type_label(v: &EditorVal) -> &'static str {
    match v {
        EditorVal::Auto => "Auto",
        EditorVal::Px(_) => "Px",
        EditorVal::Percent(_) => "Percent",
        EditorVal::Vw(_) => "Vw",
        EditorVal::Vh(_) => "Vh",
    }
}

fn val_editor(ui: &mut egui::Ui, label: &str, val: &mut EditorVal, dirty: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        let current = val_type_label(val);
        egui::ComboBox::from_id_salt(format!("val_{label}"))
            .selected_text(current)
            .show_ui(ui, |ui| {
                let variants = [
                    ("Auto", EditorVal::Auto),
                    ("Px", EditorVal::Px(inner_f32(val))),
                    ("Percent", EditorVal::Percent(inner_f32(val))),
                    ("Vw", EditorVal::Vw(inner_f32(val))),
                    ("Vh", EditorVal::Vh(inner_f32(val))),
                ];
                for (name, variant) in variants {
                    if ui.selectable_label(val_type_label(val) == name, name).clicked() {
                        *val = variant;
                        *dirty = true;
                    }
                }
            });
        if let Some(inner) = val_inner_mut(val) {
            if ui.add(egui::DragValue::new(inner).speed(1.0)).changed() {
                *dirty = true;
            }
        }
    });
}

fn inner_f32(v: &EditorVal) -> f32 {
    match v {
        EditorVal::Auto => 0.0,
        EditorVal::Px(x) | EditorVal::Percent(x) | EditorVal::Vw(x) | EditorVal::Vh(x) => *x,
    }
}

fn val_inner_mut(v: &mut EditorVal) -> Option<&mut f32> {
    match v {
        EditorVal::Auto => None,
        EditorVal::Px(x) | EditorVal::Percent(x) | EditorVal::Vw(x) | EditorVal::Vh(x) => {
            Some(x)
        }
    }
}

fn rect_editor(
    ui: &mut egui::Ui,
    prefix: &str,
    rect: &mut crate::model::EditorRect,
    dirty: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label("T:");
        val_editor(ui, &format!("{prefix}_t"), &mut rect.top, dirty);
        ui.label("R:");
        val_editor(ui, &format!("{prefix}_r"), &mut rect.right, dirty);
    });
    ui.horizontal(|ui| {
        ui.label("B:");
        val_editor(ui, &format!("{prefix}_b"), &mut rect.bottom, dirty);
        ui.label("L:");
        val_editor(ui, &format!("{prefix}_l"), &mut rect.left, dirty);
    });
}

fn combo_flex_dir(ui: &mut egui::Ui, val: &mut EditorFlexDirection, dirty: &mut bool) {
    let options = [
        EditorFlexDirection::Row,
        EditorFlexDirection::Column,
        EditorFlexDirection::RowReverse,
        EditorFlexDirection::ColumnReverse,
    ];
    egui::ComboBox::from_id_salt("flex_dir")
        .selected_text(val.to_string())
        .show_ui(ui, |ui| {
            for opt in &options {
                if ui.selectable_value(val, *opt, opt.to_string()).changed() {
                    *dirty = true;
                }
            }
        });
}

fn combo_justify(ui: &mut egui::Ui, val: &mut EditorJustifyContent, dirty: &mut bool) {
    let options = [
        EditorJustifyContent::FlexStart,
        EditorJustifyContent::FlexEnd,
        EditorJustifyContent::Center,
        EditorJustifyContent::SpaceBetween,
        EditorJustifyContent::SpaceAround,
        EditorJustifyContent::SpaceEvenly,
    ];
    egui::ComboBox::from_id_salt("justify")
        .selected_text(val.to_string())
        .show_ui(ui, |ui| {
            for opt in &options {
                if ui.selectable_value(val, *opt, opt.to_string()).changed() {
                    *dirty = true;
                }
            }
        });
}

fn combo_align(ui: &mut egui::Ui, val: &mut EditorAlignItems, dirty: &mut bool) {
    let options = [
        EditorAlignItems::FlexStart,
        EditorAlignItems::FlexEnd,
        EditorAlignItems::Center,
        EditorAlignItems::Stretch,
        EditorAlignItems::Baseline,
    ];
    egui::ComboBox::from_id_salt("align")
        .selected_text(val.to_string())
        .show_ui(ui, |ui| {
            for opt in &options {
                if ui.selectable_value(val, *opt, opt.to_string()).changed() {
                    *dirty = true;
                }
            }
        });
}

fn combo_position(ui: &mut egui::Ui, val: &mut EditorPositionType, dirty: &mut bool) {
    let options = [EditorPositionType::Relative, EditorPositionType::Absolute];
    egui::ComboBox::from_id_salt("position")
        .selected_text(val.to_string())
        .show_ui(ui, |ui| {
            for opt in &options {
                if ui.selectable_value(val, *opt, opt.to_string()).changed() {
                    *dirty = true;
                }
            }
        });
}

fn combo_overflow(ui: &mut egui::Ui, id: &str, val: &mut EditorOverflow, dirty: &mut bool) {
    let options = [EditorOverflow::Visible, EditorOverflow::Clip, EditorOverflow::Hidden];
    egui::ComboBox::from_id_salt(id)
        .selected_text(val.to_string())
        .show_ui(ui, |ui| {
            for opt in &options {
                if ui.selectable_value(val, *opt, opt.to_string()).changed() {
                    *dirty = true;
                }
            }
        });
}
