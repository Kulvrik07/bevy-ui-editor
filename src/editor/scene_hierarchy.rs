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
        SceneNodeKind::Model(_) => "🎨",
        SceneNodeKind::Camera => "📷",
        SceneNodeKind::AudioSource(_) => "🔊",
    }
}

fn node_matches_filter(snap: &NodeSnapshot, filter: &str) -> bool {
    if snap.name.to_lowercase().contains(filter) { return true; }
    snap.children.iter().any(|c| node_matches_filter(c, filter))
}

fn node_icon_color(kind: &SceneNodeKind) -> Color32 {
    match kind {
        SceneNodeKind::Empty => Color32::from_rgb(180, 180, 180),
        SceneNodeKind::Mesh(_) => Color32::from_rgb(100, 180, 255),
        SceneNodeKind::Light(_) => Color32::from_rgb(255, 220, 80),
        SceneNodeKind::Model(_) => Color32::from_rgb(100, 210, 190),
        SceneNodeKind::Camera => Color32::from_rgb(180, 120, 255),
        SceneNodeKind::AudioSource(_) => Color32::from_rgb(100, 220, 100),
    }
}

// ─── Context menu actions ─────────────────────────────────────────────────────

enum HierarchyAction {
    Select(u64),
    ToggleVisibility(u64),
    Delete(u64),
    Duplicate(u64),
    Unparent(u64),
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
    mut editor: ResMut<crate::model::EditorState>,
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

            // Search / filter
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("🔍").size(12.0));
                ui.add(egui::TextEdit::singleline(&mut editor.hierarchy_filter)
                    .hint_text("Filter nodes...")
                    .desired_width(ui.available_width() - 16.0));
            });
            ui.add_space(2.0);

            let filter = editor.hierarchy_filter.to_lowercase();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(4.0);
                for snap in &snaps {
                    if !filter.is_empty() && !node_matches_filter(snap, &filter) {
                        continue;
                    }
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
                let (drop_rect, _drop_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
                if dragging_id.is_some() {
                    let over_bottom = ui.input(|i| {
                        i.pointer.hover_pos().is_some_and(|p| drop_rect.contains(p))
                    });
                    if over_bottom {
                        ui.painter().rect_filled(drop_rect, 0.0, Color32::from_rgba_premultiplied(80, 140, 255, 30));
                        if let Some(last) = snaps.last() {
                            new_drop_target = Some(DropTarget {
                                target_id: last.id,
                                position: DropPosition::After,
                            });
                        }
                    }
                }
            });
        });

    // Detect drag end via pointer release (widget-based drag_stopped is unreliable
    // because allocate_exact_size uses auto-generated IDs that change each frame)
    if drag_drop.dragging.is_some() && !drag_ended {
        let released = egui_ctx.input(|i| i.pointer.any_released());
        if released {
            drag_ended = true;
        }
    }

    // Paint drag overlay at cursor position
    if let Some(drag_id) = dragging_id {
        if let Some(pointer_pos) = egui_ctx.pointer_hover_pos() {
            let drag_name = find_name_in_snaps(&snaps, drag_id).unwrap_or("Node");
            let text = format!("  ↕ {drag_name}  ");
            let painter = egui_ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag_overlay")));
            let font = egui::FontId::proportional(12.0);
            let text_color = Color32::from_rgb(200, 200, 220);
            // Estimate text size (approx 7px per char at 12pt)
            let text_width = text.len() as f32 * 7.0;
            let text_height = 18.0;
            let rect = egui::Rect::from_min_size(
                pointer_pos + egui::vec2(12.0, -10.0),
                egui::vec2(text_width + 8.0, text_height + 4.0),
            );
            painter.rect_filled(rect, 4.0, Color32::from_rgb(45, 55, 75));
            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, Color32::from_rgb(80, 140, 255)), egui::StrokeKind::Outside);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &text,
                font,
                text_color,
            );
        }
    }

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
            HierarchyAction::Unparent(id) => {
                // Move node out of its parent to root level
                undo.push_snapshot(&doc.nodes);
                if let Some(node) = doc.take_node(id) {
                    doc.nodes.push(node);
                    changed.dirty = true;
                    console.info(format!("Unparented node {id} to root"));
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

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn find_name_in_snaps<'a>(snaps: &'a [NodeSnapshot], id: u64) -> Option<&'a str> {
    for s in snaps {
        if s.id == id { return Some(&s.name); }
        if let Some(found) = find_name_in_snaps(&s.children, id) {
            return Some(found);
        }
    }
    None
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
    let row_height = 22.0;

    let id = egui::Id::new(("hier_node", snap.id));
    let has_children = !snap.children.is_empty();

    // Allocate the full row as a single interactive rect — no text selection possible
    let (row_rect, row_resp) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), row_height),
        Sense::click_and_drag(),
    );

    // Use raw pointer position for drop detection (hovered() doesn't work during drags)
    let is_drop_candidate = dragging_id.is_some() && !is_being_dragged;
    let pointer_over_row = is_drop_candidate && ui.input(|i| {
        i.pointer.hover_pos().is_some_and(|p| row_rect.contains(p))
    });
    if pointer_over_row {
        let mouse_y = ui.input(|i| i.pointer.hover_pos().map(|p| p.y).unwrap_or(row_rect.center().y));
        let frac = (mouse_y - row_rect.top()) / row_height;
        if frac < 0.25 {
            *drop_target = Some(DropTarget { target_id: snap.id, position: DropPosition::Before });
        } else if frac > 0.75 && !has_children {
            *drop_target = Some(DropTarget { target_id: snap.id, position: DropPosition::After });
        } else {
            *drop_target = Some(DropTarget { target_id: snap.id, position: DropPosition::Inside });
        }
    }

    // Background
    let is_parent_drop = drop_target.as_ref().is_some_and(|dt| dt.target_id == snap.id && dt.position == DropPosition::Inside);
    let row_bg = if is_parent_drop {
        Color32::from_rgb(50, 80, 140)
    } else if is_selected {
        Color32::from_rgb(50, 70, 110)
    } else if is_being_dragged {
        Color32::from_rgba_premultiplied(60, 60, 60, 100)
    } else if row_resp.hovered() && dragging_id.is_none() {
        Color32::from_rgb(42, 42, 48)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(row_rect, 2.0, row_bg);

    // Parent-drop border highlight
    if is_parent_drop {
        ui.painter().rect_stroke(row_rect.shrink(1.0), 2.0, egui::Stroke::new(1.5, Color32::from_rgb(80, 140, 255)), egui::StrokeKind::Inside);
    }

    let text_x = row_rect.left() + 8.0 + indent;
    let center_y = row_rect.center().y;

    // Collapse arrow
    if has_children {
        let cs = CollapsingState::load_with_default_open(ui.ctx(), id, true);
        let arrow = if cs.is_open() { "▾" } else { "▸" };
        ui.painter().text(
            egui::pos2(text_x, center_y),
            egui::Align2::LEFT_CENTER,
            arrow,
            egui::FontId::proportional(10.0),
            Color32::from_rgb(130, 130, 140),
        );
    }
    let content_x = text_x + 14.0;

    // Visibility eye
    let vis_icon = if snap.visible { "👁" } else { "🚫" };
    let vis_color = if snap.visible {
        Color32::from_rgb(140, 140, 140)
    } else {
        Color32::from_rgb(70, 70, 70)
    };
    let eye_rect = egui::Rect::from_center_size(
        egui::pos2(content_x + 6.0, center_y),
        Vec2::new(16.0, row_height),
    );
    ui.painter().text(
        eye_rect.center(),
        egui::Align2::CENTER_CENTER,
        vis_icon,
        egui::FontId::proportional(11.0),
        vis_color,
    );

    // Icon
    let icon_x = content_x + 20.0;
    ui.painter().text(
        egui::pos2(icon_x, center_y),
        egui::Align2::LEFT_CENTER,
        node_icon(&snap.kind),
        egui::FontId::proportional(12.0),
        node_icon_color(&snap.kind),
    );

    // Name text (painted, not a widget — no text selection)
    let name_x = icon_x + 18.0;
    let name_color = if is_being_dragged {
        Color32::from_rgb(140, 140, 140)
    } else if snap.visible {
        Color32::from_rgb(220, 220, 220)
    } else {
        Color32::from_rgb(100, 100, 100)
    };
    ui.painter().text(
        egui::pos2(name_x, center_y),
        egui::Align2::LEFT_CENTER,
        &snap.name,
        egui::FontId::proportional(13.0),
        name_color,
    );

    // Interaction
    if row_resp.drag_started() {
        *drag_started = Some(snap.id);
    }
    if row_resp.drag_stopped() {
        *drag_ended = true;
    }

    // Click on eye area = toggle visibility, else select
    if row_resp.clicked() {
        let click_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or_default());
        if eye_rect.contains(click_pos) {
            actions.push(HierarchyAction::ToggleVisibility(snap.id));
        } else if has_children {
            // Click on arrow area toggles collapse
            let arrow_rect = egui::Rect::from_min_size(
                egui::pos2(text_x - 2.0, row_rect.top()),
                Vec2::new(14.0, row_height),
            );
            if arrow_rect.contains(click_pos) {
                let mut cs = CollapsingState::load_with_default_open(ui.ctx(), id, true);
                cs.toggle(ui);
                cs.store(ui.ctx());
            } else {
                actions.push(HierarchyAction::Select(snap.id));
            }
        } else {
            actions.push(HierarchyAction::Select(snap.id));
        }
    }

    // Context menu
    row_resp.context_menu(|ui| {
        if ui.button("🗑 Delete").clicked() {
            actions.push(HierarchyAction::Delete(snap.id));
            ui.close();
        }
        if ui.button("📋 Duplicate").clicked() {
            actions.push(HierarchyAction::Duplicate(snap.id));
            ui.close();
        }
        ui.separator();
        if ui.button("⬆ Unparent (move to root)").clicked() {
            actions.push(HierarchyAction::Unparent(snap.id));
            ui.close();
        }
        ui.separator();
        ui.label(RichText::new("Add Child").strong().size(11.0));
        add_object_menu(ui, Some(snap.id), actions);
    });

    // Children (with collapsing)
    if has_children {
        let cs = CollapsingState::load_with_default_open(ui.ctx(), id, true);
        if cs.is_open() {
            for child in &snap.children {
                draw_node_tree(
                    ui, child, sel_id, depth + 1, dragging_id, drop_target,
                    drag_started, drag_ended, actions,
                );
            }
        }
    }

    // Drop indicator lines (painted based on current drop_target, not separate widgets)
    if let Some(dt) = drop_target.as_ref() {
        if dt.target_id == snap.id {
            let left = row_rect.left() + 8.0 + indent;
            let right = row_rect.right() - 4.0;
            match dt.position {
                DropPosition::Before => {
                    let y = row_rect.top();
                    ui.painter().line_segment(
                        [egui::pos2(left, y), egui::pos2(right, y)],
                        egui::Stroke::new(2.0, Color32::from_rgb(80, 140, 255)),
                    );
                    ui.painter().circle_filled(egui::pos2(left, y), 3.0, Color32::from_rgb(80, 140, 255));
                }
                DropPosition::After => {
                    let y = row_rect.bottom();
                    ui.painter().line_segment(
                        [egui::pos2(left, y), egui::pos2(right, y)],
                        egui::Stroke::new(2.0, Color32::from_rgb(80, 140, 255)),
            );
                    ui.painter().circle_filled(egui::pos2(left, y), 3.0, Color32::from_rgb(80, 140, 255));
                }
                DropPosition::Inside => {} // handled by row background
            }
        }
    }
}
