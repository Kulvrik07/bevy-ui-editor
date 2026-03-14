use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::export::generate_rust_code;
use crate::model::{
    EditorChanged, EditorDocument, EditorIdCounter, EditorNodeType, EditorSelection, EditorUiNode,
    ShowExportWindow,
};

fn new_node(id: u64, node_type: EditorNodeType) -> EditorUiNode {
    let name = match node_type {
        EditorNodeType::Container => "Container",
        EditorNodeType::Text => "Text",
        EditorNodeType::Button => "Button",
        EditorNodeType::Image => "Image",
    };
    EditorUiNode {
        id,
        name: name.to_string(),
        node_type,
        ..Default::default()
    }
}

pub fn toolbar_system(
    mut contexts: EguiContexts,
    mut document: ResMut<EditorDocument>,
    mut selection: ResMut<EditorSelection>,
    mut id_counter: ResMut<EditorIdCounter>,
    mut changed: ResMut<EditorChanged>,
    mut export_window: ResMut<ShowExportWindow>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🎨 Bevy UI Editor");
            ui.separator();

            let mut add_type: Option<EditorNodeType> = None;
            if ui.button("+ Container").clicked() { add_type = Some(EditorNodeType::Container); }
            if ui.button("+ Text").clicked() { add_type = Some(EditorNodeType::Text); }
            if ui.button("+ Button").clicked() { add_type = Some(EditorNodeType::Button); }
            if ui.button("+ Image").clicked() { add_type = Some(EditorNodeType::Image); }

            if let Some(nt) = add_type {
                let id = id_counter.next_id();
                let node = new_node(id, nt);
                let parent_id = selection.selected;
                document.add_child(parent_id, node);
                selection.selected = Some(id);
                changed.dirty = true;
            }

            ui.separator();

            if ui
                .add_enabled(
                    selection.selected.is_some(),
                    egui::Button::new("🗑 Delete"),
                )
                .clicked()
            {
                if let Some(sel) = selection.selected {
                    document.remove_node(sel);
                    selection.selected = None;
                    changed.dirty = true;
                }
            }

            ui.separator();

            if ui.button("📤 Export Rust Code").clicked() {
                let code = generate_rust_code(&document);
                export_window.code = code;
                export_window.show = true;
            }
        });
    });
}
