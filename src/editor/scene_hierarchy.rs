use bevy::prelude::*;
use bevy_egui::egui::{self, collapsing_header::CollapsingState, Color32, Frame, Margin, RichText, Sense, Vec2};
use bevy_egui::EguiContexts;

use crate::model::{
    ConsoleLog, DragDropState, DropPosition, DropTarget, SceneChanged, SceneDocument, SceneIdCounter,
    SceneLightKind, SceneNode, SceneNodeKind, ScenePrimitive, SceneSelection, UndoHistory, new_scene_node,
};

// ─── Snapshot for safe iteration ──────────────────────────────────────────────

#[derive(Clone)]
struct NodeSnapshot {
    id: u64,
    name: String,
    kind: SceneNodeKind,
    visible: bool,
    children: Vec<NodeSnapshot>,
}

fn snapshot_nodes(nodes: &[SceneNode]) -> Vec<NodeSnapshot> {
    nodes
        .iter()
        .map(|n| NodeSnapshot {
            id: n.id,
            name: n.name.clone(),
            kind: n.kind.clone(),
            visible: n.visible,
            children: snapshot_nodes(&n.children),
        })
        .collect()
}

// ─── Icon helper ──────────────────────────────────────────────────────────────

fn node_icon(kind: &SceneNodeKind) -> &'static str {
    match kind {
        SceneNodeKind::Empty => "○",
        SceneNodeKind::Mesh(_) => "◆",
        SceneNodeKind::Light(_) => "☀",
    }
}

fn node_icon_color(kind: &SceneNodeKind) -> Color32 {
    match kind {
        SceneNodeKind::Empty => Color32::from_rgb(180, 180, 180),
        SceneNodeKind::Mesh(_) => Color32::from_rgb(100, 180, 255),
        SceneNodeKind::Light(_) => Color32::from_rgb(255, 220, 80),
    }
}

// ─── Context menu actions ─────────────────────────────────────────────────────

enum HierarchyAction {
    Select(u64),
    ToggleVisibility(u64),
    Delete(u64),
    Duplicate(u64),
    AddChild(u64, SceneNodeKind),
    AddRoot(SceneNodeKind),
}

// ─── Main system ──────────────────────────────────────────────────────────────

pub fn scene_hierarchy_system(
    mut ctx: EguiContexts,
    mut doc: ResMut<SceneDocument>,
    mut selection: ResMut<SceneSelection>,
    mut changed: ResMut<SceneChanged>,
    mut id_counter: ResMut<SceneIdCounter>,
    mut drag_drop: ResMut<DragDropState>,
    mut undo: ResMut<UndoHistory>,
    mut console: ResMut<ConsoleLog>,
    app_mode: Res<super::launcher::AppModeRes>,
    editor: Res<crate::model::EditorState>,
) {
    if app_mode.mode != super::launcher::AppMode::Editor { return; }
    if editor.play_mode { return; }
    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    let snaps = snapshot_nodes(&doc.nodes);
    let sel_id = selection.selected;
    let mut actions: Vec<HierarchyAction> = Vec::new();

    // Drag-drop state from resource
    let dragging_id = drag_drop.dragging;
    let mut new_drop_target: Option<DropTarget> = None;
    let mut drag_started: Option<u64> = None;
    let mut drag_ended = false;

    egui::SidePanel::left("scene_hierarchy")
        .default_width(220.0)
        .min_width(180.0)
        .max_width(400.0)
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
                ui.label(RichText::new("Scene Hierarchy").strong().color(Color32::from_rgb(200, 200, 200)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.menu_button(
                        RichText::new("＋ Add").color(Color32::from_rgb(100, 200, 100)).size(11.0),
                        |ui| {
                            ui.set_min_width(180.0);
                            add_object_menu(ui, None, &mut actions);
                        },
                    );
                });
            });
            ui.add_space(2.0);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                for snap in &snaps {
                    draw_node_tree(
                        ui,
                        snap,
                        sel_id,
                        0,
                        dragging_id,
                        &mut new_drop_target,
                        &mut drag_started,
                        &mut drag_ended,
                        &mut actions,
                    );
                }
                ui.add_space(4.0);

                // Drop zone at the very bottom: drop as root last child
                let (drop_rect, drop_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
                if dragging_id.is_some() && drop_resp.hovered() {
                    ui.painter().rect_filled(drop_rect, 0.0, Color32::from_rgba_premultiplied(80, 140, 255, 30));
                    // Dropping on empty space = append to root
                    if let Some(last) = snaps.last() {
                        new_drop_target = Some(DropTarget {
                            target_id: last.id,
                            position: DropPosition::After,
                        });
                    }
                }
            });
        });

    // Apply drag-drop state
    if let Some(id) = drag_started {
        drag_drop.dragging = Some(id);
    }

    if drag_ended {
        // Perform the move
        if let (Some(node_id), Some(target)) = (drag_drop.dragging, &new_drop_target) {
            if node_id != target.target_id {
                undo.push_snapshot(&doc.nodes);
                if doc.move_node(node_id, target.target_id, target.position) {
                    changed.dirty = true;
                    console.info(format!("Moved node {node_id}"));
                }
            }
        }
        drag_drop.dragging = None;
        drag_drop.drop_target = None;
    } else {
        drag_drop.drop_target = new_drop_target;
    }

    // Process actions
    for action in actions {
        match action {
            HierarchyAction::Select(id) => {
                selection.selected = Some(id);
            }
            HierarchyAction::ToggleVisibility(id) => {
                if let Some(node) = doc.find_node_mut(id) {
                    node.visible = !node.visible;
                    changed.dirty = true;
                }
            }
            HierarchyAction::Delete(id) => {
                undo.push_snapshot(&doc.nodes);
                doc.remove_node(id);
                if selection.selected == Some(id) {
                    selection.selected = None;
                }
                changed.dirty = true;
                console.info(format!("Deleted node {id}"));
            }
            HierarchyAction::Duplicate(id) => {
                if let Some(node) = doc.find_node(id) {
                    let mut cloned = node.clone();
                    fn reassign_ids(n: &mut SceneNode, counter: &mut SceneIdCounter) {
                        n.id = counter.next_id();
                        for c in &mut n.children {
                            reassign_ids(c, counter);
                        }
                    }
                    reassign_ids(&mut cloned, &mut id_counter);
                    cloned.name = format!("{} (copy)", cloned.name);
                    undo.push_snapshot(&doc.nodes);
                    doc.nodes.push(cloned);
                    changed.dirty = true;
                    console.info(format!("Duplicated node {id}"));
                }
            }
            HierarchyAction::AddChild(parent_id, kind) => {
                let new_id = id_counter.next_id();
                let child = new_scene_node(new_id, kind);
                undo.push_snapshot(&doc.nodes);
                doc.add_node(Some(parent_id), child);
                changed.dirty = true;
                console.info(format!("Added child to node {parent_id}"));
            }
            HierarchyAction::AddRoot(kind) => {
                let new_id = id_counter.next_id();
                let node = new_scene_node(new_id, kind);
                let name = node.name.clone();
                undo.push_snapshot(&doc.nodes);
                doc.add_node(None, node);
                selection.selected = Some(new_id);
                changed.dirty = true;
                console.info(format!("Added {name}"));
            }
        }
    }
}

// ─── Add object submenu ──────────────────────────────────────────────────────

fn add_object_menu(
    ui: &mut egui::Ui,
    parent_id: Option<u64>,
    actions: &mut Vec<HierarchyAction>,
) {
    let action = |parent: Option<u64>, kind: SceneNodeKind| -> HierarchyAction {
        match parent {
            Some(pid) => HierarchyAction::AddChild(pid, kind),
            None => HierarchyAction::AddRoot(kind),
        }
    };

    // Empty node
    if ui.button("  ○  Empty").clicked() {
        actions.push(action(parent_id, SceneNodeKind::Empty));
        ui.close();
    }

    ui.separator();
    ui.label(RichText::new("3D Meshes").strong().size(10.0).color(Color32::from_rgb(100, 180, 255)));

    let meshes: &[(&str, ScenePrimitive)] = &[
        ("  ◆  Cube", ScenePrimitive::Cube),
        ("  ●  Sphere", ScenePrimitive::Sphere),
        ("  ▮  Cylinder", ScenePrimitive::Cylinder),
        ("  ⬬  Capsule", ScenePrimitive::Capsule),
        ("  ▬  Plane", ScenePrimitive::Plane),
        ("  ◎  Torus", ScenePrimitive::Torus),
        ("  △  Cone", ScenePrimitive::Cone),
        ("  ◇  Tetrahedron", ScenePrimitive::Tetrahedron),
    ];
    for (label, prim) in meshes {
        if ui.button(*label).clicked() {
            actions.push(action(parent_id, SceneNodeKind::Mesh(*prim)));
            ui.close();
        }
    }

    ui.separator();
    ui.label(RichText::new("Lights").strong().size(10.0).color(Color32::from_rgb(255, 220, 80)));

    let lights: &[(&str, SceneLightKind)] = &[
        ("  ☀  Point Light", SceneLightKind::Point),
        ("  ◐  Directional Light", SceneLightKind::Directional),
        ("  ◉  Spot Light", SceneLightKind::Spot),
    ];
    for (label, lk) in lights {
        if ui.button(*label).clicked() {
            actions.push(action(parent_id, SceneNodeKind::Light(*lk)));
            ui.close();
        }
    }
}

// ─── Recursive tree drawing ──────────────────────────────────────────────────

fn draw_node_tree(
    ui: &mut egui::Ui,
    snap: &NodeSnapshot,
    sel_id: Option<u64>,
    depth: usize,
    dragging_id: Option<u64>,
    drop_target: &mut Option<DropTarget>,
    drag_started: &mut Option<u64>,
    drag_ended: &mut bool,
    actions: &mut Vec<HierarchyAction>,
) {
    let is_selected = sel_id == Some(snap.id);
    let is_being_dragged = dragging_id == Some(snap.id);
    let indent = depth as f32 * 16.0;

    let id = egui::Id::new(("hier_node", snap.id));

    // Drop indicator BEFORE
    if dragging_id.is_some() && !is_being_dragged {
        let (line_rect, line_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 3.0), Sense::hover());
        if line_resp.hovered() {
            ui.painter().rect_filled(line_rect, 1.0, Color32::from_rgb(80, 140, 255));
            *drop_target = Some(DropTarget {
                target_id: snap.id,
                position: DropPosition::Before,
            });
        }
    }

    // Main row
    let row_bg = if is_selected {
        Color32::from_rgb(50, 70, 110)
    } else if is_being_dragged {
        Color32::from_rgba_premultiplied(80, 80, 80, 128)
    } else {
        Color32::TRANSPARENT
    };

    let has_children = !snap.children.is_empty();

    Frame::NONE
        .fill(row_bg)
        .inner_margin(Margin::symmetric(0, 1))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0 + indent);

                // Collapse arrow (or spacer)
                if has_children {
                    // We use CollapsingState for persistent open/close
                    // But draw our own row for the interactive bits
                } else {
                    ui.add_space(14.0);
                }

                // Visibility eye
                let vis_icon = if snap.visible { "👁" } else { "🚫" };
                let vis_color = if snap.visible {
                    Color32::from_rgb(160, 160, 160)
                } else {
                    Color32::from_rgb(80, 80, 80)
                };
                if ui.add(egui::Label::new(RichText::new(vis_icon).color(vis_color).size(11.0)).sense(Sense::click())).clicked() {
                    actions.push(HierarchyAction::ToggleVisibility(snap.id));
                }

                // Icon
                ui.label(RichText::new(node_icon(&snap.kind)).color(node_icon_color(&snap.kind)).size(12.0));

                // Name label (draggable + clickable)
                let label_text = RichText::new(&snap.name)
                    .color(if snap.visible {
                        Color32::from_rgb(220, 220, 220)
                    } else {
                        Color32::from_rgb(100, 100, 100)
                    })
                    .size(13.0);

                let resp = ui.add(egui::Label::new(label_text).sense(Sense::click_and_drag()));

                // Drag start
                if resp.drag_started() {
                    *drag_started = Some(snap.id);
                }
                if resp.drag_stopped() {
                    *drag_ended = true;
                }

                // Click to select
                if resp.clicked() {
                    actions.push(HierarchyAction::Select(snap.id));
                }

                // Drop-on-self = Inside
                if dragging_id.is_some() && !is_being_dragged && resp.hovered() {
                    *drop_target = Some(DropTarget {
                        target_id: snap.id,
                        position: DropPosition::Inside,
                    });
                }

                // Context menu
                resp.context_menu(|ui| {
                    if ui.button("🗑 Delete").clicked() {
                        actions.push(HierarchyAction::Delete(snap.id));
                        ui.close();
                    }
                    if ui.button("📋 Duplicate").clicked() {
                        actions.push(HierarchyAction::Duplicate(snap.id));
                        ui.close();
                    }
                    ui.separator();
                    ui.label(RichText::new("Add Child").strong().size(11.0));
                    add_object_menu(ui, Some(snap.id), actions);
                });
            });
        });

    // Children (with collapsing)
    if has_children {
        CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_body_unindented(ui, |ui| {
                for child in &snap.children {
                    draw_node_tree(
                        ui, child, sel_id, depth + 1, dragging_id, drop_target,
                        drag_started, drag_ended, actions,
                    );
                }
            });
    }

    // Drop indicator AFTER
    if dragging_id.is_some() && !is_being_dragged {
        let (line_rect, line_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 3.0), Sense::hover());
        if line_resp.hovered() {
            ui.painter().rect_filled(line_rect, 1.0, Color32::from_rgb(80, 140, 255));
            *drop_target = Some(DropTarget {
                target_id: snap.id,
                position: DropPosition::After,
            });
        }
    }
}
