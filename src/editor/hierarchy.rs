use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::model::{
    EditorChanged, EditorDocument, EditorIdCounter, EditorNodeType, EditorSelection, EditorUiNode,
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

pub fn hierarchy_system(
    mut contexts: EguiContexts,
    mut document: ResMut<EditorDocument>,
    mut selection: ResMut<EditorSelection>,
    mut id_counter: ResMut<EditorIdCounter>,
    mut changed: ResMut<EditorChanged>,
) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::SidePanel::left("hierarchy_panel")
        .exact_width(250.0)
        .show(ctx, |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 110.0)
                .show(ui, |ui| {
                    // Collect node rendering data without borrowing document mutably
                    let roots_snapshot: Vec<(u64, String, EditorNodeType)> =
                        document.roots.iter().map(|n| (n.id, n.name.clone(), n.node_type)).collect();

                    let mut action: Option<HierarchyAction> = None;
                    let selected = selection.selected;

                    for (id, name, ntype) in &roots_snapshot {
                        render_node_tree(ui, *id, name, *ntype, selected, &document, &mut action);
                    }

                    if let Some(act) = action {
                        apply_action(act, &mut document, &mut selection, &mut id_counter, &mut changed);
                    }
                });

            ui.separator();

            // Add buttons
            ui.horizontal_wrapped(|ui| {
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
            });

            if ui.add_enabled(selection.selected.is_some(), egui::Button::new("🗑 Delete Selected")).clicked() {
                if let Some(sel) = selection.selected {
                    document.remove_node(sel);
                    selection.selected = None;
                    changed.dirty = true;
                }
            }
        });
}

enum HierarchyAction {
    Select(u64),
}

fn render_node_tree(
    ui: &mut egui::Ui,
    id: u64,
    name: &str,
    ntype: EditorNodeType,
    selected: Option<u64>,
    document: &EditorDocument,
    action: &mut Option<HierarchyAction>,
) {
    let label = format!("[{}] {}", ntype, name);
    let is_selected = selected == Some(id);

    // Check if this node has children
    let has_children = document
        .find_node(id)
        .map(|n| !n.children.is_empty())
        .unwrap_or(false);

    if has_children {
        let header = egui::CollapsingHeader::new(&label)
            .id_salt(id)
            .default_open(true)
            .show(ui, |ui| {
                // Render children
                if let Some(node) = document.find_node(id) {
                    let children_snapshot: Vec<(u64, String, EditorNodeType)> = node
                        .children
                        .iter()
                        .map(|c| (c.id, c.name.clone(), c.node_type))
                        .collect();
                    for (cid, cname, ctype) in children_snapshot {
                        render_node_tree(ui, cid, &cname, ctype, selected, document, action);
                    }
                }
            });

        if header.header_response.clicked() {
            *action = Some(HierarchyAction::Select(id));
        }
        if is_selected {
            // Highlight the header
            ui.painter().rect_stroke(
                header.header_response.rect,
                2.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 180, 255)),
                egui::StrokeKind::Middle,
            );
        }
    } else {
        let response = ui.selectable_label(is_selected, &label);
        if response.clicked() {
            *action = Some(HierarchyAction::Select(id));
        }
    }
}

fn apply_action(
    action: HierarchyAction,
    _document: &mut EditorDocument,
    selection: &mut EditorSelection,
    _id_counter: &mut EditorIdCounter,
    _changed: &mut EditorChanged,
) {
    match action {
        HierarchyAction::Select(id) => {
            selection.selected = Some(id);
        }
    }
}
