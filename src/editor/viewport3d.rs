use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::window::PrimaryWindow;
use bevy_camera::Viewport;
use bevy_egui::{egui, EguiContexts};

use crate::model::{
    ConsoleLog, EditorState, SceneChanged, SceneDocument, SceneNode, SceneNodeKind,
    ScenePrimitive, SceneLightKind, SceneSelection, TransformMode, UndoHistory,
};

// ─── Orbit camera ─────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct OrbitCamera;

#[derive(Resource)]
pub struct OrbitState {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub focus: Vec3,
}

impl Default for OrbitState {
    fn default() -> Self {
        OrbitState {
            yaw: 45.0_f32.to_radians(),
            pitch: 30.0_f32.to_radians(),
            distance: 12.0,
            focus: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

// ─── Components for synced entities ───────────────────────────────────────────

#[derive(Component)]
pub struct SceneEntity(pub u64);

#[derive(Component)]
pub struct SceneRoot;

// ─── Viewport rect (central area after all panels) ───────────────────────────

#[derive(Resource, Default)]
pub struct ViewportRect {
    pub rect: Option<egui::Rect>,
}

// ─── Drag axis for per-axis transforms ────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragAxis {
    X,
    Y,
    Z,
    All,
}

// ─── Viewport transform drag state ───────────────────────────────────────────

#[derive(Resource)]
pub struct ViewportDragState {
    pub active: bool,
    pub mode: TransformMode,
    pub axis: Option<DragAxis>,
    pub start_mouse: Vec2,
    pub start_value: [f32; 3],
    pub snapshot_pushed: bool,
}

impl Default for ViewportDragState {
    fn default() -> Self {
        ViewportDragState {
            active: false,
            mode: TransformMode::Select,
            axis: None,
            start_mouse: Vec2::ZERO,
            start_value: [0.0; 3],
            snapshot_pushed: false,
        }
    }
}

// ─── Info bar at the bottom of the viewport ───────────────────────────────────

pub fn viewport_info_system(
    mut contexts: EguiContexts,
    document: Res<SceneDocument>,
    selection: Res<SceneSelection>,
    editor: Res<EditorState>,
    app_mode: Res<crate::editor::launcher::AppModeRes>,
) {
    if app_mode.mode != crate::editor::launcher::AppMode::Editor { return; }
    if editor.play_mode { return; }
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    egui::TopBottomPanel::bottom("viewport_info")
        .exact_height(24.0)
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 30, 35))
                .inner_margin(egui::Margin::symmetric(8, 3)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let count = document.collect_ids().len();
                ui.label(
                    egui::RichText::new(format!("Objects: {count}"))
                        .small()
                        .color(egui::Color32::from_rgb(160, 160, 170)),
                );
                ui.separator();

                // Show current mode
                let mode_text = format!("Mode: {}", editor.transform_mode);
                ui.label(
                    egui::RichText::new(mode_text)
                        .small()
                        .color(egui::Color32::from_rgb(200, 180, 100)),
                );
                ui.separator();

                if let Some(id) = selection.selected {
                    if let Some(node) = document.find_node(id) {
                        ui.label(
                            egui::RichText::new(format!("Selected: {} ({})", node.name, node.kind))
                                .small()
                                .color(egui::Color32::from_rgb(100, 180, 255)),
                        );
                        ui.separator();
                        let [x, y, z] = node.translation;
                        ui.label(
                            egui::RichText::new(format!("Pos: ({x:.1}, {y:.1}, {z:.1})"))
                                .small()
                                .color(egui::Color32::from_rgb(140, 140, 150)),
                        );
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No selection")
                            .small()
                            .color(egui::Color32::from_rgb(100, 100, 110)),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("LMB: Select | W: Move | E: Rotate | R: Scale | RMB: Orbit | MMB: Pan")
                            .small()
                            .color(egui::Color32::from_rgb(80, 80, 90)),
                    );
                });
            });
        });
}

// ─── Capture remaining viewport rect after all panels ─────────────────────────

pub fn viewport_rect_system(
    mut contexts: EguiContexts,
    mut viewport_rect: ResMut<ViewportRect>,
    app_mode: Res<crate::editor::launcher::AppModeRes>,
    editor: Res<EditorState>,
) {
    if app_mode.mode != crate::editor::launcher::AppMode::Editor { return; }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if editor.play_mode {
        // In play mode, viewport is entire screen
        viewport_rect.rect = Some(ctx.available_rect());
    } else {
        viewport_rect.rect = Some(ctx.available_rect());
    }
}

// ─── Apply viewport rect to Camera3d ─────────────────────────────────────────

pub fn apply_viewport_rect_system(
    viewport_rect: Res<ViewportRect>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_q: Query<&mut Camera, With<OrbitCamera>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok(mut camera) = camera_q.single_mut() else { return };

    if let Some(rect) = viewport_rect.rect {
        let scale = window.scale_factor();
        let phys_x = (rect.left() * scale) as u32;
        let phys_y = (rect.top() * scale) as u32;
        let phys_w = ((rect.width() * scale) as u32).max(1);
        let phys_h = ((rect.height() * scale) as u32).max(1);
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(phys_x, phys_y),
            physical_size: UVec2::new(phys_w, phys_h),
            ..default()
        });
    }
}

// ─── Camera orbit system ──────────────────────────────────────────────────────

pub fn camera_orbit_system(
    mut orbit: ResMut<OrbitState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    mut camera_q: Query<&mut Transform, With<OrbitCamera>>,
    mut contexts: EguiContexts,
    editor: Res<EditorState>,
) {
    if editor.play_mode { return; }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };
    let egui_wants = ctx.wants_pointer_input() || ctx.is_pointer_over_area();

    if !egui_wants {
        let sensitivity = 0.005;
        let pan_sensitivity = 0.01;
        let delta = accumulated_motion.delta;

        // Right-click: orbit
        if mouse_button.pressed(MouseButton::Right) {
            orbit.yaw -= delta.x * sensitivity;
            orbit.pitch = (orbit.pitch - delta.y * sensitivity).clamp(-1.4, 1.4);
        }
        // Middle-click: pan
        else if mouse_button.pressed(MouseButton::Middle) {
            let dist = orbit.distance;
            let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin());
            let up = Vec3::Y;
            orbit.focus -= right * delta.x * pan_sensitivity * dist * 0.1;
            orbit.focus += up * delta.y * pan_sensitivity * dist * 0.1;
        }

        // Scroll: zoom
        let scroll_y = accumulated_scroll.delta.y;
        if scroll_y.abs() > 0.0 {
            orbit.distance = (orbit.distance - scroll_y * orbit.distance * 0.1).clamp(0.5, 100.0);
        }
    }

    // Apply orbit to camera transform
    let eye = orbit.focus
        + Vec3::new(
            orbit.distance * orbit.pitch.cos() * orbit.yaw.sin(),
            orbit.distance * orbit.pitch.sin(),
            orbit.distance * orbit.pitch.cos() * orbit.yaw.cos(),
        );

    for mut transform in camera_q.iter_mut() {
        *transform = Transform::from_translation(eye).looking_at(orbit.focus, Vec3::Y);
    }
}

// ─── Click-to-select & viewport transforms ────────────────────────────────────

pub fn viewport_interact_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<OrbitCamera>>,
    scene_entities: Query<(&SceneEntity, &GlobalTransform)>,
    mut selection: ResMut<SceneSelection>,
    mut doc: ResMut<SceneDocument>,
    mut changed: ResMut<SceneChanged>,
    mut drag: ResMut<ViewportDragState>,
    mut undo: ResMut<UndoHistory>,
    editor: Res<EditorState>,
    orbit: Res<OrbitState>,
    mut ctx: EguiContexts,
) {
    if editor.play_mode { return; }

    let egui_wants = ctx.ctx_mut().map(|c| c.wants_pointer_input() || c.is_pointer_over_area()).unwrap_or(true);

    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_global)) = camera_q.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else {
        if drag.active { drag.active = false; }
        return;
    };

    // ── Click to select or initiate axis drag ────────────────────────
    if mouse_button.just_pressed(MouseButton::Left) && !egui_wants && !drag.active {
        if let Ok(ray) = camera.viewport_to_world(cam_global, cursor_pos) {
            // First check if we're clicking a gizmo handle (per-axis)
            let mut clicked_axis: Option<DragAxis> = None;

            if let Some(sel_id) = selection.selected {
                if editor.transform_mode != TransformMode::Select {
                    if let Some(node) = doc.find_node(sel_id) {
                        let pos = Vec3::from(node.translation);
                        let node_scale = Vec3::from(node.scale);
                        let axis_len = node_scale.max_element().max(0.5) * 1.5 + 0.8;
                        // Scale hit radii with camera distance for consistent screen-space feel
                        let cam_dist = (ray.origin - pos).length().max(1.0);
                        let dist_scale = (cam_dist * 0.04).clamp(0.8, 3.0);
                        let handle_radius = 0.45 * dist_scale;

                        match editor.transform_mode {
                            TransformMode::Translate | TransformMode::Scale => {
                                // Test axis handle tips (spheres at the end of each axis)
                                let tips = [
                                    (pos + Vec3::X * axis_len, DragAxis::X),
                                    (pos + Vec3::Y * axis_len, DragAxis::Y),
                                    (pos + Vec3::Z * axis_len, DragAxis::Z),
                                ];
                                let mut best_dist = f32::MAX;
                                for (tip_pos, axis) in &tips {
                                    if let Some(d) = ray_sphere_intersect(ray.origin, ray.direction.into(), *tip_pos, handle_radius) {
                                        if d < best_dist {
                                            best_dist = d;
                                            clicked_axis = Some(*axis);
                                        }
                                    }
                                }
                                // Also test along the axis lines (thin cylinder approximation via multiple spheres)
                                if clicked_axis.is_none() {
                                    let line_radius = 0.25 * dist_scale;
                                    let axes = [
                                        (Vec3::X, DragAxis::X),
                                        (Vec3::Y, DragAxis::Y),
                                        (Vec3::Z, DragAxis::Z),
                                    ];
                                    let mut best_line_dist = f32::MAX;
                                    for (dir, axis) in &axes {
                                        for i in 1..=16 {
                                            let t = i as f32 / 16.0 * axis_len;
                                            let sample = pos + *dir * t;
                                            if let Some(d) = ray_sphere_intersect(ray.origin, ray.direction.into(), sample, line_radius) {
                                                if d < best_line_dist {
                                                    best_line_dist = d;
                                                    clicked_axis = Some(*axis);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            TransformMode::Rotate => {
                                // Test clicks near the rotation circles
                                let ring_r = axis_len * 0.7;
                                let ring_thickness = 0.35 * dist_scale;
                                let ring_samples = 48;
                                let normals: [(Vec3, Vec3, DragAxis); 3] = [
                                    (Vec3::Y, Vec3::Z, DragAxis::X), // YZ circle → rotate X
                                    (Vec3::X, Vec3::Z, DragAxis::Y), // XZ circle → rotate Y
                                    (Vec3::X, Vec3::Y, DragAxis::Z), // XY circle → rotate Z
                                ];
                                let mut best_dist = f32::MAX;
                                for (a1, a2, axis) in &normals {
                                    for i in 0..ring_samples {
                                        let angle = i as f32 / ring_samples as f32 * std::f32::consts::TAU;
                                        let sample = pos + (*a1 * angle.cos() + *a2 * angle.sin()) * ring_r;
                                        if let Some(d) = ray_sphere_intersect(ray.origin, ray.direction.into(), sample, ring_thickness) {
                                            if d < best_dist {
                                                best_dist = d;
                                                clicked_axis = Some(*axis);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }

                        // Also test center for "all axes" drag
                        if clicked_axis.is_none() {
                            if let Some(_) = ray_sphere_intersect(ray.origin, ray.direction.into(), pos, handle_radius * 2.0) {
                                clicked_axis = Some(DragAxis::All);
                            }
                        }
                    }
                }
            }

            if let Some(axis) = clicked_axis {
                // Start per-axis drag
                if let Some(sel_id) = selection.selected {
                    if let Some(node) = doc.find_node(sel_id) {
                        drag.active = true;
                        drag.mode = editor.transform_mode;
                        drag.axis = Some(axis);
                        drag.start_mouse = cursor_pos;
                        drag.snapshot_pushed = false;
                        drag.start_value = match editor.transform_mode {
                            TransformMode::Translate => node.translation,
                            TransformMode::Rotate => node.rotation_euler,
                            TransformMode::Scale => node.scale,
                            TransformMode::Select => [0.0; 3],
                        };
                    }
                }
            } else {
                // No gizmo hit — try to select an object
                let mut best_id: Option<u64> = None;
                let mut best_dist = f32::MAX;

                for (scene_ent, global_tf) in &scene_entities {
                    let ent_pos = global_tf.translation();
                    let radius = get_pick_radius(&doc, scene_ent.0);
                    let dist = ray_sphere_intersect(ray.origin, ray.direction.into(), ent_pos, radius);
                    if let Some(d) = dist {
                        if d < best_dist {
                            best_dist = d;
                            best_id = Some(scene_ent.0);
                        }
                    }
                }

                if let Some(id) = best_id {
                    selection.selected = Some(id);
                } else {
                    selection.selected = None;
                }
            }
        }
    }

    // ── Continue drag ────────────────────────────────────────────────
    if drag.active && mouse_button.pressed(MouseButton::Left) {
        if let Some(sel) = selection.selected {
            let mouse_delta = accumulated_motion.delta;

            if !drag.snapshot_pushed && mouse_delta.length() > 0.5 {
                undo.push_snapshot(&doc.nodes);
                drag.snapshot_pushed = true;
            }

            let sensitivity = orbit.distance * 0.003;
            let snap_t = if editor.snap_enabled { editor.snap_translate } else { 0.0 };
            let snap_r = if editor.snap_enabled { editor.snap_rotate } else { 0.0 };
            let snap_s = if editor.snap_enabled { editor.snap_scale } else { 0.0 };

            if let Some(node) = doc.find_node_mut(sel) {
                let pos = Vec3::from(node.translation);

                match drag.mode {
                    TransformMode::Translate => {
                        match drag.axis {
                            Some(DragAxis::X) => {
                                let delta = project_mouse_to_axis(camera, cam_global, pos, Vec3::X, mouse_delta, sensitivity);
                                node.translation[0] += delta;
                                if snap_t > 0.0 { node.translation[0] = snap_value(node.translation[0], snap_t); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Y) => {
                                let delta = project_mouse_to_axis(camera, cam_global, pos, Vec3::Y, mouse_delta, sensitivity);
                                node.translation[1] += delta;
                                if snap_t > 0.0 { node.translation[1] = snap_value(node.translation[1], snap_t); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Z) => {
                                let delta = project_mouse_to_axis(camera, cam_global, pos, Vec3::Z, mouse_delta, sensitivity);
                                node.translation[2] += delta;
                                if snap_t > 0.0 { node.translation[2] = snap_value(node.translation[2], snap_t); }
                                changed.dirty = true;
                            }
                            _ => {
                                let cam_right = cam_global.right();
                                let cam_up = cam_global.up();
                                let world_delta = cam_right * mouse_delta.x * sensitivity
                                    + cam_up * (-mouse_delta.y) * sensitivity;
                                node.translation[0] += world_delta.x;
                                node.translation[1] += world_delta.y;
                                node.translation[2] += world_delta.z;
                                if snap_t > 0.0 {
                                    node.translation[0] = snap_value(node.translation[0], snap_t);
                                    node.translation[1] = snap_value(node.translation[1], snap_t);
                                    node.translation[2] = snap_value(node.translation[2], snap_t);
                                }
                                changed.dirty = true;
                            }
                        }
                    }
                    TransformMode::Rotate => {
                        match drag.axis {
                            Some(DragAxis::X) => {
                                node.rotation_euler[0] += mouse_delta.y * 0.5;
                                if snap_r > 0.0 { node.rotation_euler[0] = snap_value(node.rotation_euler[0], snap_r); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Y) => {
                                node.rotation_euler[1] += mouse_delta.x * 0.5;
                                if snap_r > 0.0 { node.rotation_euler[1] = snap_value(node.rotation_euler[1], snap_r); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Z) => {
                                node.rotation_euler[2] += mouse_delta.x * 0.5;
                                if snap_r > 0.0 { node.rotation_euler[2] = snap_value(node.rotation_euler[2], snap_r); }
                                changed.dirty = true;
                            }
                            _ => {
                                node.rotation_euler[1] += mouse_delta.x * 0.5;
                                node.rotation_euler[0] += mouse_delta.y * 0.5;
                                if snap_r > 0.0 {
                                    node.rotation_euler[0] = snap_value(node.rotation_euler[0], snap_r);
                                    node.rotation_euler[1] = snap_value(node.rotation_euler[1], snap_r);
                                }
                                changed.dirty = true;
                            }
                        }
                    }
                    TransformMode::Scale => {
                        match drag.axis {
                            Some(DragAxis::X) => {
                                node.scale[0] = (node.scale[0] + mouse_delta.x * 0.005).max(0.01);
                                if snap_s > 0.0 { node.scale[0] = snap_value(node.scale[0], snap_s).max(0.01); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Y) => {
                                node.scale[1] = (node.scale[1] - mouse_delta.y * 0.005).max(0.01);
                                if snap_s > 0.0 { node.scale[1] = snap_value(node.scale[1], snap_s).max(0.01); }
                                changed.dirty = true;
                            }
                            Some(DragAxis::Z) => {
                                node.scale[2] = (node.scale[2] + mouse_delta.x * 0.005).max(0.01);
                                if snap_s > 0.0 { node.scale[2] = snap_value(node.scale[2], snap_s).max(0.01); }
                                changed.dirty = true;
                            }
                            _ => {
                                let scale_delta = mouse_delta.x * 0.005;
                                node.scale[0] = (node.scale[0] + scale_delta).max(0.01);
                                node.scale[1] = (node.scale[1] + scale_delta).max(0.01);
                                node.scale[2] = (node.scale[2] + scale_delta).max(0.01);
                                if snap_s > 0.0 {
                                    node.scale[0] = snap_value(node.scale[0], snap_s).max(0.01);
                                    node.scale[1] = snap_value(node.scale[1], snap_s).max(0.01);
                                    node.scale[2] = snap_value(node.scale[2], snap_s).max(0.01);
                                }
                                changed.dirty = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // ── End drag ─────────────────────────────────────────────────────
    if drag.active && mouse_button.just_released(MouseButton::Left) {
        drag.active = false;
        drag.axis = None;
    }

    // ── Cancel drag with Escape ──────────────────────────────────────
    if drag.active && keys.just_pressed(KeyCode::Escape) {
        if let Some(sel) = selection.selected {
            if let Some(node) = doc.find_node_mut(sel) {
                match drag.mode {
                    TransformMode::Translate => node.translation = drag.start_value,
                    TransformMode::Rotate => node.rotation_euler = drag.start_value,
                    TransformMode::Scale => node.scale = drag.start_value,
                    _ => {}
                }
                changed.dirty = true;
            }
        }
        if drag.snapshot_pushed {
            undo.undo_stack.pop();
        }
        drag.active = false;
        drag.axis = None;
    }
}

// ─── Project mouse movement onto a world axis (screen-space projection) ──────

fn project_mouse_to_axis(
    camera: &Camera,
    cam_global: &GlobalTransform,
    pos: Vec3,
    axis: Vec3,
    mouse_delta: Vec2,
    sensitivity: f32,
) -> f32 {
    // Project the axis direction to screen space
    let Ok(origin_screen) = camera.world_to_viewport(cam_global, pos) else {
        return mouse_delta.x * sensitivity;
    };
    let Ok(axis_screen) = camera.world_to_viewport(cam_global, pos + axis) else {
        return mouse_delta.x * sensitivity;
    };
    let axis_dir_2d = axis_screen - origin_screen;
    if axis_dir_2d.length() < 0.001 {
        return 0.0;
    }
    let axis_dir_2d = axis_dir_2d.normalize();
    let projected = mouse_delta.dot(axis_dir_2d);
    projected * sensitivity
}

fn snap_value(v: f32, grid: f32) -> f32 {
    (v / grid).round() * grid
}

fn reassign_ids_recursive(node: &mut SceneNode, base: u64) {
    let mut offset = 1u64;
    for child in &mut node.children {
        child.id = base.wrapping_add(offset * 100);
        reassign_ids_recursive(child, child.id);
        offset += 1;
    }
}

// ─── Pick radius helper ──────────────────────────────────────────────────────

fn get_pick_radius(doc: &SceneDocument, id: u64) -> f32 {
    if let Some(node) = doc.find_node(id) {
        match &node.kind {
            SceneNodeKind::Empty => 0.3,
            SceneNodeKind::Mesh(ScenePrimitive::Plane) => {
                node.scale[0].max(node.scale[2]) * 0.5
            }
            SceneNodeKind::Mesh(ScenePrimitive::Sphere) => {
                node.scale[0] * 0.5
            }
            SceneNodeKind::Mesh(_) => {
                let s = node.scale;
                s[0].max(s[1]).max(s[2]) * 0.6
            }
            SceneNodeKind::Light(_) => 0.5,
            SceneNodeKind::Model(_) => {
                let s = node.scale;
                s[0].max(s[1]).max(s[2]) * 1.0
            }
            SceneNodeKind::Camera => 0.4,
            SceneNodeKind::AudioSource(_) => 0.4,
        }
    } else {
        0.5
    }
}

// ─── Ray-sphere intersection ─────────────────────────────────────────────────

fn ray_sphere_intersect(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let a = direction.dot(direction);
    let b = 2.0 * oc.dot(direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / (2.0 * a);
    if t > 0.0 {
        Some(t)
    } else {
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        if t2 > 0.0 { Some(t2) } else { None }
    }
}

// ─── Mesh helper ──────────────────────────────────────────────────────────────

fn mesh_for_primitive(meshes: &mut ResMut<Assets<Mesh>>, primitive: &ScenePrimitive) -> Handle<Mesh> {
    match primitive {
        ScenePrimitive::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        ScenePrimitive::Sphere => meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap()),
        ScenePrimitive::Cylinder => meshes.add(Cylinder::new(0.5, 1.0)),
        ScenePrimitive::Capsule => meshes.add(Capsule3d::new(0.35, 1.0)),
        ScenePrimitive::Plane => meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5))),
        ScenePrimitive::Torus => meshes.add(Torus::new(0.3, 0.5)),
        ScenePrimitive::Cone => meshes.add(Cone::new(0.5, 1.0)),
        ScenePrimitive::Tetrahedron => meshes.add(Tetrahedron::default()),
    }
}

fn material_for_node(materials: &mut ResMut<Assets<StandardMaterial>>, node: &SceneNode) -> Handle<StandardMaterial> {
    use crate::model::SceneAlphaMode;
    let [r, g, b, a] = node.color;
    let [er, eg, eb, ea] = node.emissive;
    let alpha_mode = match node.alpha_mode {
        SceneAlphaMode::Opaque => AlphaMode::Opaque,
        SceneAlphaMode::Blend => AlphaMode::Blend,
        SceneAlphaMode::Mask => AlphaMode::Mask(node.alpha_cutoff),
        SceneAlphaMode::AlphaToCoverage => AlphaMode::AlphaToCoverage,
    };
    materials.add(StandardMaterial {
        base_color: Color::srgba(r, g, b, a),
        metallic: node.metallic,
        perceptual_roughness: node.roughness,
        emissive: LinearRgba::new(er * 5.0, eg * 5.0, eb * 5.0, ea),
        unlit: node.unlit,
        double_sided: node.double_sided,
        alpha_mode,
        ..default()
    })
}

// ─── Scene sync ───────────────────────────────────────────────────────────────

pub fn scene_sync_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut changed: ResMut<SceneChanged>,
    document: Res<SceneDocument>,
    selection: Res<SceneSelection>,
    existing: Query<Entity, With<SceneRoot>>,
    asset_server: Res<AssetServer>,
    editor: Res<EditorState>,
) {
    if !changed.dirty {
        return;
    }
    changed.dirty = false;

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    for node in &document.nodes {
        spawn_scene_node(&mut commands, &mut meshes, &mut materials, node, selection.selected, &asset_server, editor.play_mode);
    }
}

fn spawn_scene_node(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node: &SceneNode,
    selected: Option<u64>,
    asset_server: &Res<AssetServer>,
    play_mode: bool,
) {
    let [rx, ry, rz] = node.rotation_euler;
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        rx.to_radians(),
        ry.to_radians(),
        rz.to_radians(),
    );
    let transform = Transform {
        translation: Vec3::from(node.translation),
        rotation,
        scale: Vec3::from(node.scale),
    };

    let vis = if node.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    match &node.kind {
        SceneNodeKind::Empty => {
            let ec = commands.spawn((
                transform,
                vis,
                SceneRoot,
                SceneEntity(node.id),
            ));
            let entity_id = ec.id();
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
        SceneNodeKind::Mesh(primitive) => {
            let mesh_handle = mesh_for_primitive(meshes, primitive);
            let mat = material_for_node(materials, node);

            let mut ec = commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(mat),
                transform,
                vis,
                SceneRoot,
                SceneEntity(node.id),
            ));

            if play_mode {
                use bevy_rapier3d::prelude::*;
                let rigid_body = match node.physics_body {
                    crate::model::PhysicsBodyType::Static => RigidBody::Fixed,
                    crate::model::PhysicsBodyType::Dynamic => RigidBody::Dynamic,
                    crate::model::PhysicsBodyType::Kinematic => RigidBody::KinematicPositionBased,
                    crate::model::PhysicsBodyType::None => RigidBody::Fixed,
                };
                let collider = match node.collider_shape {
                    crate::model::ColliderShape::Sphere => Collider::ball(node.scale[0] * 0.5),
                    crate::model::ColliderShape::Capsule => Collider::capsule_y(node.scale[1] * 0.5, node.scale[0] * 0.5),
                    crate::model::ColliderShape::Cylinder => Collider::cylinder(node.scale[1] * 0.5, node.scale[0] * 0.5),
                    _ => Collider::cuboid(node.scale[0] * 0.5, node.scale[1] * 0.5, node.scale[2] * 0.5),
                };
                if node.physics_body != crate::model::PhysicsBodyType::None {
                    ec.insert((rigid_body, collider));
                }
            }

            let entity_id = ec.id();
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
        SceneNodeKind::Light(light_kind) => {
            let [lr, lg, lb, _] = node.light_color;
            let color = Color::srgb(lr, lg, lb);
            match light_kind {
                SceneLightKind::Point => {
                    commands.spawn((
                        PointLight {
                            color,
                            intensity: node.light_intensity,
                            range: node.light_range,
                            shadows_enabled: node.light_shadows,
                            ..default()
                        },
                        transform, vis, SceneRoot, SceneEntity(node.id),
                    ));
                }
                SceneLightKind::Directional => {
                    commands.spawn((
                        DirectionalLight {
                            color,
                            illuminance: node.light_intensity,
                            shadows_enabled: node.light_shadows,
                            ..default()
                        },
                        transform, vis, SceneRoot, SceneEntity(node.id),
                    ));
                }
                SceneLightKind::Spot => {
                    commands.spawn((
                        SpotLight {
                            color,
                            intensity: node.light_intensity,
                            range: node.light_range,
                            outer_angle: node.spot_angle.to_radians(),
                            inner_angle: (node.spot_angle * 0.8).to_radians(),
                            shadows_enabled: node.light_shadows,
                            ..default()
                        },
                        transform, vis, SceneRoot, SceneEntity(node.id),
                    ));
                }
            }
            let entity_id = commands.spawn(()).id(); // dummy, not used
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
        SceneNodeKind::Model(ref model_path) => {
            let path_owned = model_path.clone();
            let scene_handle: Handle<bevy::scene::Scene> = asset_server.load(
                GltfAssetLabel::Scene(0).from_asset(path_owned),
            );
            let ec = commands.spawn((
                bevy::scene::SceneRoot(scene_handle),
                transform,
                vis,
                SceneRoot,
                SceneEntity(node.id),
            ));
            let entity_id = ec.id();
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
        SceneNodeKind::Camera => {
            let ec = commands.spawn((
                transform,
                vis,
                SceneRoot,
                SceneEntity(node.id),
            ));
            let entity_id = ec.id();
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
        SceneNodeKind::AudioSource(_) => {
            let ec = commands.spawn((
                transform,
                vis,
                SceneRoot,
                SceneEntity(node.id),
            ));
            let entity_id = ec.id();
            for child in &node.children {
                spawn_scene_child(commands, meshes, materials, child, entity_id, selected, asset_server, play_mode);
            }
        }
    }
}

fn spawn_scene_child(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    node: &SceneNode,
    parent: Entity,
    selected: Option<u64>,
    asset_server: &Res<AssetServer>,
    play_mode: bool,
) {
    let [rx, ry, rz] = node.rotation_euler;
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        rx.to_radians(),
        ry.to_radians(),
        rz.to_radians(),
    );
    let transform = Transform {
        translation: Vec3::from(node.translation),
        rotation,
        scale: Vec3::from(node.scale),
    };
    let vis = if node.visible { Visibility::Visible } else { Visibility::Hidden };
    let mut child_entity = Entity::PLACEHOLDER;

    match &node.kind {
        SceneNodeKind::Empty => {
            commands.entity(parent).with_children(|cb| {
                child_entity = cb.spawn((transform, vis, SceneEntity(node.id))).id();
            });
        }
        SceneNodeKind::Mesh(primitive) => {
            let mesh_handle = mesh_for_primitive(meshes, primitive);
            let mat = material_for_node(materials, node);
            commands.entity(parent).with_children(|cb| {
                child_entity = cb.spawn((
                    Mesh3d(mesh_handle), MeshMaterial3d(mat), transform, vis, SceneEntity(node.id),
                )).id();
            });
        }
        SceneNodeKind::Light(light_kind) => {
            let [lr, lg, lb, _] = node.light_color;
            let color = Color::srgb(lr, lg, lb);
            commands.entity(parent).with_children(|cb| {
                match light_kind {
                    SceneLightKind::Point => {
                        child_entity = cb.spawn((
                            PointLight { color, intensity: node.light_intensity, range: node.light_range, shadows_enabled: node.light_shadows, ..default() },
                            transform, vis, SceneEntity(node.id),
                        )).id();
                    }
                    SceneLightKind::Directional => {
                        child_entity = cb.spawn((
                            DirectionalLight { color, illuminance: node.light_intensity, shadows_enabled: node.light_shadows, ..default() },
                            transform, vis, SceneEntity(node.id),
                        )).id();
                    }
                    SceneLightKind::Spot => {
                        child_entity = cb.spawn((
                            SpotLight { color, intensity: node.light_intensity, range: node.light_range, outer_angle: node.spot_angle.to_radians(), inner_angle: (node.spot_angle * 0.8).to_radians(), shadows_enabled: node.light_shadows, ..default() },
                            transform, vis, SceneEntity(node.id),
                        )).id();
                    }
                }
            });
        }
        SceneNodeKind::Model(ref model_path) => {
            let path_owned = model_path.clone();
            let scene_handle: Handle<bevy::scene::Scene> = asset_server.load(
                GltfAssetLabel::Scene(0).from_asset(path_owned),
            );
            commands.entity(parent).with_children(|cb| {
                child_entity = cb.spawn((
                    bevy::scene::SceneRoot(scene_handle),
                    transform, vis, SceneEntity(node.id),
                )).id();
            });
        }
        SceneNodeKind::Camera => {
            commands.entity(parent).with_children(|cb| {
                child_entity = cb.spawn((transform, vis, SceneEntity(node.id))).id();
            });
        }
        SceneNodeKind::AudioSource(_) => {
            commands.entity(parent).with_children(|cb| {
                child_entity = cb.spawn((transform, vis, SceneEntity(node.id))).id();
            });
        }
    }

    for grandchild in &node.children {
        spawn_scene_child(commands, meshes, materials, grandchild, child_entity, selected, asset_server, play_mode);
    }
}

// ─── Selection gizmo with transform handles ──────────────────────────────────

pub fn selection_gizmo_system(
    mut gizmos: Gizmos,
    selection: Res<SceneSelection>,
    scene_entities: Query<(&SceneEntity, &GlobalTransform)>,
    document: Res<SceneDocument>,
    editor: Res<EditorState>,
    drag: Res<ViewportDragState>,
) {
    if editor.play_mode { return; }
    let Some(sel_id) = selection.selected else { return };

    for (scene_ent, global_tf) in &scene_entities {
        if scene_ent.0 != sel_id { continue; }
        let pos = global_tf.translation();

        let size = if let Some(node) = document.find_node(sel_id) {
            match &node.kind {
                SceneNodeKind::Mesh(ScenePrimitive::Cube) => Vec3::from(node.scale) * 0.5,
                SceneNodeKind::Mesh(ScenePrimitive::Sphere) => Vec3::splat(node.scale[0] * 0.5),
                SceneNodeKind::Mesh(ScenePrimitive::Plane) => Vec3::new(node.scale[0] * 0.5, 0.01, node.scale[2] * 0.5),
                _ => Vec3::splat(0.5) * Vec3::from(node.scale),
            }
        } else {
            Vec3::splat(0.5)
        };

        // Wireframe selection box
        let sel_color = Color::srgba(0.2, 0.6, 1.0, 0.6);
        gizmos.cube(
            Transform::from_translation(pos).with_scale(size * 2.2),
            sel_color,
        );

        // Axis colors (highlight active axis during drag)
        let active_axis = drag.axis;
        let x_color = if active_axis == Some(DragAxis::X) {
            Color::srgb(1.0, 0.8, 0.2)
        } else {
            Color::srgb(1.0, 0.2, 0.2)
        };
        let y_color = if active_axis == Some(DragAxis::Y) {
            Color::srgb(1.0, 0.8, 0.2)
        } else {
            Color::srgb(0.2, 1.0, 0.2)
        };
        let z_color = if active_axis == Some(DragAxis::Z) {
            Color::srgb(1.0, 0.8, 0.2)
        } else {
            Color::srgb(0.3, 0.3, 1.0)
        };

        let axis_len = size.max_element().max(0.5) * 1.5 + 0.8;

        match editor.transform_mode {
            TransformMode::Translate => {
                // Axis lines
                gizmos.line(pos, pos + Vec3::X * axis_len, x_color);
                gizmos.line(pos, pos + Vec3::Y * axis_len, y_color);
                gizmos.line(pos, pos + Vec3::Z * axis_len, z_color);

                // Arrow heads (cone approximation with 3 lines each)
                let tip_size = 0.15;
                let tip_x = pos + Vec3::X * axis_len;
                let tip_y = pos + Vec3::Y * axis_len;
                let tip_z = pos + Vec3::Z * axis_len;

                // X arrow head
                gizmos.line(tip_x, tip_x - Vec3::X * tip_size + Vec3::Y * tip_size * 0.5, x_color);
                gizmos.line(tip_x, tip_x - Vec3::X * tip_size - Vec3::Y * tip_size * 0.5, x_color);
                gizmos.line(tip_x, tip_x - Vec3::X * tip_size + Vec3::Z * tip_size * 0.5, x_color);
                gizmos.line(tip_x, tip_x - Vec3::X * tip_size - Vec3::Z * tip_size * 0.5, x_color);

                // Y arrow head
                gizmos.line(tip_y, tip_y - Vec3::Y * tip_size + Vec3::X * tip_size * 0.5, y_color);
                gizmos.line(tip_y, tip_y - Vec3::Y * tip_size - Vec3::X * tip_size * 0.5, y_color);
                gizmos.line(tip_y, tip_y - Vec3::Y * tip_size + Vec3::Z * tip_size * 0.5, y_color);
                gizmos.line(tip_y, tip_y - Vec3::Y * tip_size - Vec3::Z * tip_size * 0.5, y_color);

                // Z arrow head
                gizmos.line(tip_z, tip_z - Vec3::Z * tip_size + Vec3::X * tip_size * 0.5, z_color);
                gizmos.line(tip_z, tip_z - Vec3::Z * tip_size - Vec3::X * tip_size * 0.5, z_color);
                gizmos.line(tip_z, tip_z - Vec3::Z * tip_size + Vec3::Y * tip_size * 0.5, z_color);
                gizmos.line(tip_z, tip_z - Vec3::Z * tip_size - Vec3::Y * tip_size * 0.5, z_color);

                // Small plane handles at quarter-length (XY, XZ, YZ planes)
                let plane_offset = axis_len * 0.3;
                let plane_size = axis_len * 0.12;
                // XY plane handle
                gizmos.line(pos + Vec3::X * plane_offset, pos + Vec3::X * plane_offset + Vec3::Y * plane_size, Color::srgba(1.0, 1.0, 0.2, 0.6));
                gizmos.line(pos + Vec3::Y * plane_offset, pos + Vec3::Y * plane_offset + Vec3::X * plane_size, Color::srgba(1.0, 1.0, 0.2, 0.6));
                // XZ plane handle
                gizmos.line(pos + Vec3::X * plane_offset, pos + Vec3::X * plane_offset + Vec3::Z * plane_size, Color::srgba(1.0, 0.2, 1.0, 0.6));
                gizmos.line(pos + Vec3::Z * plane_offset, pos + Vec3::Z * plane_offset + Vec3::X * plane_size, Color::srgba(1.0, 0.2, 1.0, 0.6));

                // Center dot
                gizmos.sphere(Isometry3d::from_translation(pos), 0.08, Color::srgb(1.0, 1.0, 1.0));
            }
            TransformMode::Rotate => {
                let r = axis_len * 0.7;
                gizmos.circle(Isometry3d::new(pos, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)), r, x_color);
                gizmos.circle(Isometry3d::new(pos, Quat::IDENTITY), r, y_color);
                gizmos.circle(Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)), r, z_color);

                // Small axis indicators
                let indicator_len = r * 0.25;
                gizmos.line(pos + Vec3::X * r, pos + Vec3::X * (r + indicator_len), x_color);
                gizmos.line(pos + Vec3::Y * r, pos + Vec3::Y * (r + indicator_len), y_color);
                gizmos.line(pos + Vec3::Z * r, pos + Vec3::Z * (r + indicator_len), z_color);

                // Center sphere
                gizmos.sphere(Isometry3d::from_translation(pos), 0.06, Color::srgb(0.8, 0.8, 0.8));
            }
            TransformMode::Scale => {
                // Axis lines
                gizmos.line(pos, pos + Vec3::X * axis_len, x_color);
                gizmos.line(pos, pos + Vec3::Y * axis_len, y_color);
                gizmos.line(pos, pos + Vec3::Z * axis_len, z_color);

                // Cubes at ends
                let cube_size = 0.12;
                gizmos.cube(Transform::from_translation(pos + Vec3::X * axis_len).with_scale(Vec3::splat(cube_size * 2.0)), x_color);
                gizmos.cube(Transform::from_translation(pos + Vec3::Y * axis_len).with_scale(Vec3::splat(cube_size * 2.0)), y_color);
                gizmos.cube(Transform::from_translation(pos + Vec3::Z * axis_len).with_scale(Vec3::splat(cube_size * 2.0)), z_color);

                // Center cube (uniform scale)
                gizmos.cube(Transform::from_translation(pos).with_scale(Vec3::splat(0.16)), Color::srgb(1.0, 1.0, 1.0));
            }
            TransformMode::Select => {
                // Simple thin axis lines
                gizmos.line(pos, pos + Vec3::X * axis_len * 0.5, x_color);
                gizmos.line(pos, pos + Vec3::Y * axis_len * 0.5, y_color);
                gizmos.line(pos, pos + Vec3::Z * axis_len * 0.5, z_color);
            }
        }

        break;
    }
}

// ─── Node type gizmos (camera frustum, audio radius) ──────────────────────────

pub fn node_type_gizmo_system(
    mut gizmos: Gizmos,
    scene_entities: Query<(&SceneEntity, &GlobalTransform)>,
    document: Res<SceneDocument>,
    editor: Res<EditorState>,
    selection: Res<SceneSelection>,
) {
    if editor.play_mode { return; }

    for (scene_ent, global_tf) in &scene_entities {
        let Some(node) = document.find_node(scene_ent.0) else { continue };
        if !node.visible { continue; }
        let pos = global_tf.translation();
        let is_selected = selection.selected == Some(scene_ent.0);

        match &node.kind {
            SceneNodeKind::Camera => {
                // Draw camera frustum wireframe
                let alpha = if is_selected { 0.9 } else { 0.5 };
                let color = Color::srgba(0.7, 0.4, 1.0, alpha);
                let rotation = global_tf.compute_transform().rotation;
                let fwd = rotation * -Vec3::Z;
                let up = rotation * Vec3::Y;
                let right = rotation * Vec3::X;

                let near = 0.3_f32;
                let far = 2.0_f32;
                let aspect = 16.0 / 9.0_f32;
                let half_fov = (node.fov * 0.5).to_radians();
                let hn = near * half_fov.tan();
                let wn = hn * aspect;
                let hf = far * half_fov.tan();
                let wf = hf * aspect;

                let nc = pos + fwd * near;
                let fc = pos + fwd * far;

                let ntl = nc + up * hn - right * wn;
                let ntr = nc + up * hn + right * wn;
                let nbl = nc - up * hn - right * wn;
                let nbr = nc - up * hn + right * wn;
                let ftl = fc + up * hf - right * wf;
                let ftr = fc + up * hf + right * wf;
                let fbl = fc - up * hf - right * wf;
                let fbr = fc - up * hf + right * wf;

                // Near plane
                gizmos.line(ntl, ntr, color);
                gizmos.line(ntr, nbr, color);
                gizmos.line(nbr, nbl, color);
                gizmos.line(nbl, ntl, color);
                // Far plane
                gizmos.line(ftl, ftr, color);
                gizmos.line(ftr, fbr, color);
                gizmos.line(fbr, fbl, color);
                gizmos.line(fbl, ftl, color);
                // Edges
                gizmos.line(ntl, ftl, color);
                gizmos.line(ntr, ftr, color);
                gizmos.line(nbl, fbl, color);
                gizmos.line(nbr, fbr, color);
            }
            SceneNodeKind::AudioSource(_) => {
                // Draw audio range sphere
                let alpha = if is_selected { 0.5 } else { 0.25 };
                let color = Color::srgba(0.3, 0.8, 0.3, alpha);
                let radius = 1.5;
                // Draw 3 circles (XY, XZ, YZ planes)
                gizmos.circle(Isometry3d::from_translation(pos), radius, color);
                gizmos.circle(
                    Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    radius, color,
                );
                gizmos.circle(
                    Isometry3d::new(pos, Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                    radius, color,
                );
            }
            _ => {}
        }
    }
}

// ─── Grid gizmo ───────────────────────────────────────────────────────────────

pub fn grid_gizmo_system(mut gizmos: Gizmos, editor: Res<EditorState>) {
    if !editor.show_grid || editor.play_mode { return; }

    let half = 10;
    let color = Color::srgba(0.3, 0.3, 0.3, 0.4);
    let color_axis = Color::srgba(0.5, 0.5, 0.5, 0.6);

    for i in -half..=half {
        let c = if i == 0 { color_axis } else { color };
        gizmos.line(
            Vec3::new(i as f32, 0.0, -half as f32),
            Vec3::new(i as f32, 0.0, half as f32),
            c,
        );
        gizmos.line(
            Vec3::new(-half as f32, 0.0, i as f32),
            Vec3::new(half as f32, 0.0, i as f32),
            c,
        );
    }
}

// ─── Keyboard shortcuts ───────────────────────────────────────────────────────

pub fn keyboard_shortcuts_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut doc: ResMut<SceneDocument>,
    mut selection: ResMut<SceneSelection>,
    mut changed: ResMut<SceneChanged>,
    mut undo: ResMut<UndoHistory>,
    mut editor: ResMut<EditorState>,
    mut orbit: ResMut<OrbitState>,
    mut console: ResMut<ConsoleLog>,
    mut ctx: EguiContexts,
) {
    // In play mode, only allow Escape to exit
    if editor.play_mode {
        if keys.just_pressed(KeyCode::Escape) {
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
        return;
    }

    let wants_keyboard = ctx.ctx_mut().map(|c| c.wants_keyboard_input()).unwrap_or(false);

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // Ctrl+Z = Undo (always, even when egui has focus)
    if ctrl && keys.just_pressed(KeyCode::KeyZ) && !keys.pressed(KeyCode::ShiftLeft) {
        if let Some(prev) = undo.undo(&doc.nodes) {
            doc.nodes = prev;
            changed.dirty = true;
            console.info("Undo");
        }
    }

    // Ctrl+Shift+Z or Ctrl+Y = Redo (always, even when egui has focus)
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        if let Some(next) = undo.redo(&doc.nodes) {
            doc.nodes = next;
            changed.dirty = true;
            console.info("Redo");
        }
    }
    if ctrl && keys.just_pressed(KeyCode::KeyZ) && keys.pressed(KeyCode::ShiftLeft) {
        if let Some(next) = undo.redo(&doc.nodes) {
            doc.nodes = next;
            changed.dirty = true;
            console.info("Redo");
        }
    }

    if wants_keyboard { return; }

    // Delete = delete selected
    if keys.just_pressed(KeyCode::Delete) {
        if let Some(sel) = selection.selected {
            undo.push_snapshot(&doc.nodes);
            doc.remove_node(sel);
            selection.selected = None;
            changed.dirty = true;
            console.info("Deleted node");
        }
    }

    // Ctrl+D = duplicate
    if ctrl && keys.just_pressed(KeyCode::KeyD) {
        if let Some(sel) = selection.selected {
            if let Some(node) = doc.find_node(sel) {
                let mut dup = node.clone();
                dup.id = sel + 10000;
                dup.name = format!("{} (copy)", dup.name);
                dup.translation[0] += 1.0;
                undo.push_snapshot(&doc.nodes);
                doc.add_node(None, dup);
                selection.selected = Some(sel + 10000);
                changed.dirty = true;
                console.info("Duplicated node");
            }
        }
    }

    // Ctrl+C = copy
    if ctrl && keys.just_pressed(KeyCode::KeyC) {
        if let Some(sel) = selection.selected {
            if let Some(node) = doc.find_node(sel) {
                editor.clipboard = Some(node.clone());
                console.info(format!("Copied {}", node.name));
            }
        }
    }

    // Ctrl+V = paste
    if ctrl && keys.just_pressed(KeyCode::KeyV) {
        if let Some(ref clip) = editor.clipboard.clone() {
            let mut pasted = clip.clone();
            pasted.id = pasted.id.wrapping_add(20000);
            pasted.name = format!("{} (pasted)", clip.name);
            pasted.translation[0] += 1.0;
            let new_id = pasted.id;
            reassign_ids_recursive(&mut pasted, new_id);
            undo.push_snapshot(&doc.nodes);
            doc.add_node(selection.selected, pasted);
            selection.selected = Some(new_id);
            changed.dirty = true;
            console.info("Pasted node");
        }
    }

    // W/E/R = transform modes (only when not Ctrl)
    if !ctrl {
        if keys.just_pressed(KeyCode::KeyQ) { editor.transform_mode = TransformMode::Select; }
        if keys.just_pressed(KeyCode::KeyW) { editor.transform_mode = TransformMode::Translate; }
        if keys.just_pressed(KeyCode::KeyE) { editor.transform_mode = TransformMode::Rotate; }
        if keys.just_pressed(KeyCode::KeyR) { editor.transform_mode = TransformMode::Scale; }
    }

    // F = focus on selected
    if keys.just_pressed(KeyCode::KeyF) {
        if let Some(sel) = selection.selected {
            if let Some(node) = doc.find_node(sel) {
                orbit.focus = Vec3::from(node.translation);
                orbit.distance = 6.0;
                console.info(format!("Focused on {}", node.name));
            }
        }
    }

    // G = toggle grid
    if keys.just_pressed(KeyCode::KeyG) && !ctrl {
        editor.show_grid = !editor.show_grid;
    }
}

// ─── Play mode camera (WASD + mouse look) ────────────────────────────────────

pub fn play_mode_camera_system(
    keys: Res<ButtonInput<KeyCode>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    editor: Res<EditorState>,
    mut camera_q: Query<&mut Transform, With<OrbitCamera>>,
) {
    if !editor.play_mode { return; }

    let Ok(mut transform) = camera_q.single_mut() else { return };

    let speed = 5.0 * time.delta_secs();
    let sprint_mult = if keys.pressed(KeyCode::ShiftLeft) { 2.5 } else { 1.0 };
    let move_speed = speed * sprint_mult;

    let forward = *transform.forward();
    let right = *transform.right();

    if keys.pressed(KeyCode::KeyW) { transform.translation += forward * move_speed; }
    if keys.pressed(KeyCode::KeyS) { transform.translation -= forward * move_speed; }
    if keys.pressed(KeyCode::KeyA) { transform.translation -= right * move_speed; }
    if keys.pressed(KeyCode::KeyD) { transform.translation += right * move_speed; }
    if keys.pressed(KeyCode::Space) { transform.translation += Vec3::Y * move_speed; }
    if keys.pressed(KeyCode::ControlLeft) { transform.translation -= Vec3::Y * move_speed; }

    // Mouse look (when right button held in play mode)
    if mouse_button.pressed(MouseButton::Right) {
        let delta = accumulated_motion.delta;
        if delta.length() > 0.0 {
            let rot_speed = 0.003;
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            yaw -= delta.x * rot_speed;
            pitch = (pitch - delta.y * rot_speed).clamp(-1.4, 1.4);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        }
    }
}

// ─── Setup (unused - camera spawned in mod.rs) ───────────────────────────────

#[allow(dead_code)]
pub fn setup_3d_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        OrbitCamera,
    ));
}
