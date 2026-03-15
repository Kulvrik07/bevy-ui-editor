use std::path::{Path, PathBuf};

use crate::model::{SceneDocument, SceneLightKind, SceneNode, SceneNodeKind, ScenePrimitive, SceneProjection};

/// Result of an export operation.
pub struct ExportResult {
    pub output_dir: PathBuf,
    pub errors: Vec<String>,
}

struct ScriptInfo {
    mod_name: String,
    component_name: String,
    update_fn: String,
    source_path: PathBuf,
}

/// Export the scene as a complete, runnable Bevy project.
pub fn export_project(
    doc: &SceneDocument,
    project_root: &Path,
    output_dir: &Path,
) -> ExportResult {
    let mut errors = Vec::new();

    let scripts = collect_scene_scripts(doc, project_root, &mut errors);

    let src_dir = output_dir.join("src");
    let scripts_dir = src_dir.join("scripts");
    if let Err(e) = std::fs::create_dir_all(&scripts_dir) {
        errors.push(format!("Failed to create output dirs: {e}"));
        return ExportResult { output_dir: output_dir.to_path_buf(), errors };
    }

    // Cargo.toml
    let cargo_toml = generate_cargo_toml(output_dir);
    if let Err(e) = std::fs::write(output_dir.join("Cargo.toml"), cargo_toml) {
        errors.push(format!("Failed to write Cargo.toml: {e}"));
    }

    // Copy scripts
    for script in &scripts {
        if script.source_path.exists() {
            if let Err(e) = std::fs::copy(
                &script.source_path,
                scripts_dir.join(format!("{}.rs", script.mod_name)),
            ) {
                errors.push(format!("Failed to copy script {}: {e}", script.mod_name));
            }
        } else {
            errors.push(format!("Script not found: {}", script.source_path.display()));
        }
    }

    // scripts/mod.rs
    let scripts_mod = generate_scripts_mod(&scripts);
    if let Err(e) = std::fs::write(scripts_dir.join("mod.rs"), scripts_mod) {
        errors.push(format!("Failed to write scripts/mod.rs: {e}"));
    }

    // main.rs
    let main_rs = generate_main_rs(doc, &scripts);
    if let Err(e) = std::fs::write(src_dir.join("main.rs"), main_rs) {
        errors.push(format!("Failed to write main.rs: {e}"));
    }

    ExportResult { output_dir: output_dir.to_path_buf(), errors }
}

// ─── Cargo.toml ───────────────────────────────────────────────────────────────

fn generate_cargo_toml(output_dir: &Path) -> String {
    let name = output_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "exported_game".into());
    let package_name: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    format!(
        r#"[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.18.1"
"#
    )
}

// ─── Script discovery ─────────────────────────────────────────────────────────

fn collect_scene_scripts(
    doc: &SceneDocument,
    project_root: &Path,
    errors: &mut Vec<String>,
) -> Vec<ScriptInfo> {
    let mut paths = Vec::new();
    gather_script_paths(&doc.nodes, &mut paths);
    paths.sort();
    paths.dedup();

    let mut scripts = Vec::new();
    for rel in &paths {
        let full = project_root.join(rel);
        let stem = Path::new(rel)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if stem.is_empty() {
            errors.push(format!("Invalid script path: {rel}"));
            continue;
        }
        let (comp, func) = if full.exists() {
            parse_script_convention(&full, &stem)
        } else {
            (to_pascal_case(&stem), format!("{stem}_update"))
        };
        scripts.push(ScriptInfo {
            mod_name: stem,
            component_name: comp,
            update_fn: func,
            source_path: full,
        });
    }
    scripts
}

fn gather_script_paths(nodes: &[SceneNode], out: &mut Vec<String>) {
    for node in nodes {
        for s in &node.scripts {
            if s.enabled {
                out.push(s.path.clone());
            }
        }
        gather_script_paths(&node.children, out);
    }
}

/// Parse a script .rs file to find the `#[derive(Component)]` struct name
/// and the `pub fn ..._update` / `..._system` function name.
fn parse_script_convention(path: &Path, fallback_stem: &str) -> (String, String) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (to_pascal_case(fallback_stem), format!("{fallback_stem}_update")),
    };

    let mut component = None;
    let mut derive_hit = false;
    let mut update_fn = None;

    for line in content.lines() {
        let t = line.trim();
        if t.contains("derive") && t.contains("Component") {
            derive_hit = true;
            continue;
        }
        if derive_hit {
            if let Some(rest) = t.strip_prefix("pub struct ") {
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if !name.is_empty() {
                    component = Some(name);
                }
            }
            derive_hit = false;
        }
        if let Some(rest) = t.strip_prefix("pub fn ") {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if name.ends_with("_update") || name.ends_with("_system") {
                update_fn = Some(name);
            }
        }
    }

    (
        component.unwrap_or_else(|| to_pascal_case(fallback_stem)),
        update_fn.unwrap_or_else(|| format!("{fallback_stem}_update")),
    )
}

// ─── scripts/mod.rs ───────────────────────────────────────────────────────────

fn generate_scripts_mod(scripts: &[ScriptInfo]) -> String {
    scripts.iter().map(|s| format!("pub mod {};\n", s.mod_name)).collect()
}

// ─── main.rs ──────────────────────────────────────────────────────────────────

fn generate_main_rs(doc: &SceneDocument, scripts: &[ScriptInfo]) -> String {
    let mut o = String::new();

    o.push_str("use bevy::prelude::*;\n\nmod scripts;\n\n");

    // fn main()
    o.push_str("fn main() {\n");
    o.push_str("    App::new()\n");
    o.push_str("        .add_plugins(DefaultPlugins)\n");
    o.push_str("        .add_systems(Startup, setup_scene)\n");
    if !scripts.is_empty() {
        o.push_str("        .add_systems(Update, (\n");
        for s in scripts {
            o.push_str(&format!("            scripts::{}::{},\n", s.mod_name, s.update_fn));
        }
        o.push_str("        ))\n");
    }
    o.push_str("        .run();\n}\n\n");

    // fn setup_scene(...)
    let has_meshes = node_tree_has(&doc.nodes, |n| matches!(n.kind, SceneNodeKind::Mesh(_)));
    let has_models = node_tree_has(&doc.nodes, |n| matches!(n.kind, SceneNodeKind::Model(_)));
    let has_audio = node_tree_has(&doc.nodes, |n| matches!(n.kind, SceneNodeKind::AudioSource(_)));
    let has_cameras = node_tree_has(&doc.nodes, |n| matches!(n.kind, SceneNodeKind::Camera));
    let needs_asset_server = has_models || has_audio;

    o.push_str("fn setup_scene(\n");
    o.push_str("    mut commands: Commands,\n");
    if has_meshes {
        o.push_str("    mut meshes: ResMut<Assets<Mesh>>,\n");
        o.push_str("    mut materials: ResMut<Assets<StandardMaterial>>,\n");
    }
    if needs_asset_server {
        o.push_str("    asset_server: Res<AssetServer>,\n");
    }
    o.push_str(") {\n");

    // Only spawn a default camera if the scene doesn't have one
    if !has_cameras {
        o.push_str("    // Camera\n");
        o.push_str("    commands.spawn((\n");
        o.push_str("        Camera3d::default(),\n");
        o.push_str("        Transform::from_xyz(8.0, 6.0, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),\n");
        o.push_str("    ));\n\n");
    }

    // Scene nodes
    for node in &doc.nodes {
        emit_spawn(&mut o, node, scripts, "commands", 1);
    }

    o.push_str("}\n");
    o
}

fn node_tree_has(nodes: &[SceneNode], pred: fn(&SceneNode) -> bool) -> bool {
    nodes.iter().any(|n| pred(n) || node_tree_has(&n.children, pred))
}

// ─── Unified spawn emitter ────────────────────────────────────────────────────

/// Emit a `spawner.spawn((...))` call, optionally chained with `.with_children(...)`.
/// `spawner` is either `"commands"` for root nodes or `"parent"` inside closures.
fn emit_spawn(
    o: &mut String,
    node: &SceneNode,
    scripts: &[ScriptInfo],
    spawner: &str,
    depth: usize,
) {
    let ind = "    ".repeat(depth);

    o.push_str(&format!("{ind}// {}\n", node.name));
    o.push_str(&format!("{ind}{spawner}.spawn((\n"));

    // Kind-specific components
    emit_kind_components(o, node, depth + 1);

    // Transform
    o.push_str(&format!("{ind}    {},\n", transform_expr(node)));

    // Script marker components
    for sr in &node.scripts {
        if !sr.enabled { continue; }
        if let Some(info) = find_script(scripts, &sr.path) {
            o.push_str(&format!("{ind}    scripts::{}::{},\n", info.mod_name, info.component_name));
        }
    }

    if node.children.is_empty() {
        // No children — close spawn
        o.push_str(&format!("{ind}));\n\n"));
    } else {
        // Chain .with_children
        o.push_str(&format!("{ind})).with_children(|parent| {{\n"));
        for child in &node.children {
            emit_spawn(o, child, scripts, "parent", depth + 1);
        }
        o.push_str(&format!("{ind}}});\n\n"));
    }
}

fn emit_kind_components(o: &mut String, node: &SceneNode, depth: usize) {
    let ind = "    ".repeat(depth);

    match &node.kind {
        SceneNodeKind::Mesh(prim) => {
            let mesh = mesh_primitive_expr(prim);
            o.push_str(&format!("{ind}Mesh3d(meshes.add({mesh})),\n"));
            o.push_str(&format!("{ind}MeshMaterial3d(materials.add({})),\n", material_expr(node)));
        }
        SceneNodeKind::Light(lk) => {
            emit_light_component(o, node, lk, depth);
        }
        SceneNodeKind::Empty => {
            o.push_str(&format!("{ind}Visibility::default(),\n"));
        }
        SceneNodeKind::Model(path) => {
            o.push_str(&format!("{ind}SceneRoot(asset_server.load(\"{path}\")),\n"));
        }
        SceneNodeKind::Camera => {
            let fov = node.fov;
            let near = node.near_clip;
            let far = node.far_clip;
            match node.projection {
                SceneProjection::Perspective => {
                    o.push_str(&format!("{ind}Camera3d::default(),\n"));
                    o.push_str(&format!("{ind}Projection::Perspective(PerspectiveProjection {{\n"));
                    o.push_str(&format!("{ind}    fov: {:.4},\n", fov.to_radians()));
                    o.push_str(&format!("{ind}    near: {near:.4},\n"));
                    o.push_str(&format!("{ind}    far: {far:.1},\n"));
                    o.push_str(&format!("{ind}    ..default()\n"));
                    o.push_str(&format!("{ind}}}),\n"));
                }
                SceneProjection::Orthographic => {
                    o.push_str(&format!("{ind}Camera3d::default(),\n"));
                    o.push_str(&format!("{ind}Projection::Orthographic(OrthographicProjection::default_3d()),\n"));
                }
            }
            if node.hdr {
                o.push_str(&format!("{ind}Camera {{ hdr: true, ..default() }},\n"));
            }
        }
        SceneNodeKind::AudioSource(ref path) => {
            o.push_str(&format!("{ind}AudioPlayer(asset_server.load(\"{path}\")),\n"));
        }
    }
}

fn emit_light_component(o: &mut String, node: &SceneNode, lk: &SceneLightKind, depth: usize) {
    let ind = "    ".repeat(depth);
    match lk {
        SceneLightKind::Point => {
            o.push_str(&format!("{ind}PointLight {{\n"));
            o.push_str(&format!("{ind}    color: {},\n", color_expr(&node.light_color)));
            o.push_str(&format!("{ind}    intensity: {:.1},\n", node.light_intensity));
            o.push_str(&format!("{ind}    range: {:.1},\n", node.light_range));
            o.push_str(&format!("{ind}    shadows_enabled: {},\n", node.light_shadows));
            o.push_str(&format!("{ind}    ..default()\n"));
            o.push_str(&format!("{ind}}},\n"));
        }
        SceneLightKind::Directional => {
            o.push_str(&format!("{ind}DirectionalLight {{\n"));
            o.push_str(&format!("{ind}    color: {},\n", color_expr(&node.light_color)));
            o.push_str(&format!("{ind}    illuminance: {:.1},\n", node.light_intensity));
            o.push_str(&format!("{ind}    shadows_enabled: {},\n", node.light_shadows));
            o.push_str(&format!("{ind}    ..default()\n"));
            o.push_str(&format!("{ind}}},\n"));
        }
        SceneLightKind::Spot => {
            o.push_str(&format!("{ind}SpotLight {{\n"));
            o.push_str(&format!("{ind}    color: {},\n", color_expr(&node.light_color)));
            o.push_str(&format!("{ind}    intensity: {:.1},\n", node.light_intensity));
            o.push_str(&format!("{ind}    range: {:.1},\n", node.light_range));
            o.push_str(&format!("{ind}    shadows_enabled: {},\n", node.light_shadows));
            o.push_str(&format!("{ind}    outer_angle: {:.4},\n", node.spot_angle.to_radians()));
            o.push_str(&format!("{ind}    ..default()\n"));
            o.push_str(&format!("{ind}}},\n"));
        }
    }
}

// ─── Expression helpers ───────────────────────────────────────────────────────

fn mesh_primitive_expr(prim: &ScenePrimitive) -> &'static str {
    match prim {
        ScenePrimitive::Cube => "Cuboid::default()",
        ScenePrimitive::Sphere => "Sphere::default()",
        ScenePrimitive::Cylinder => "Cylinder::default()",
        ScenePrimitive::Capsule => "Capsule3d::default()",
        ScenePrimitive::Plane => "Plane3d::default().mesh().size(1.0, 1.0)",
        ScenePrimitive::Torus => "Torus::default()",
        ScenePrimitive::Cone => "Cone::default()",
        ScenePrimitive::Tetrahedron => "Tetrahedron::default()",
    }
}

fn transform_expr(node: &SceneNode) -> String {
    let [tx, ty, tz] = node.translation;
    let [rx, ry, rz] = node.rotation_euler;
    let [sx, sy, sz] = node.scale;

    let has_rot = rx != 0.0 || ry != 0.0 || rz != 0.0;
    let has_scale = sx != 1.0 || sy != 1.0 || sz != 1.0;

    let mut e = format!("Transform::from_translation(Vec3::new({tx:.3}, {ty:.3}, {tz:.3}))");
    if has_rot {
        e.push_str(&format!(
            "\n            .with_rotation(Quat::from_euler(EulerRot::XYZ, {:.4}, {:.4}, {:.4}))",
            rx.to_radians(), ry.to_radians(), rz.to_radians()
        ));
    }
    if has_scale {
        e.push_str(&format!("\n            .with_scale(Vec3::new({sx:.3}, {sy:.3}, {sz:.3}))"));
    }
    e
}

fn material_expr(node: &SceneNode) -> String {
    let [r, g, b, a] = node.color;
    let has_extras = node.metallic != 0.0 || node.roughness != 0.5
        || node.emissive.iter().any(|v| *v != 0.0)
        || node.unlit || node.double_sided
        || !matches!(node.alpha_mode, crate::model::SceneAlphaMode::Opaque);

    if !has_extras {
        return format!("Color::srgba({r:.3}, {g:.3}, {b:.3}, {a:.3})");
    }

    let mut s = String::from("StandardMaterial {\n");
    s.push_str(&format!("                base_color: Color::srgba({r:.3}, {g:.3}, {b:.3}, {a:.3}),\n"));
    if node.metallic != 0.0 {
        s.push_str(&format!("                metallic: {:.3},\n", node.metallic));
    }
    if node.roughness != 0.5 {
        s.push_str(&format!("                perceptual_roughness: {:.3},\n", node.roughness));
    }
    if node.emissive.iter().any(|v| *v != 0.0) {
        let [er, eg, eb, ea] = node.emissive;
        s.push_str(&format!("                emissive: LinearRgba::new({er:.3}, {eg:.3}, {eb:.3}, {ea:.3}),\n"));
    }
    if node.unlit {
        s.push_str("                unlit: true,\n");
    }
    if node.double_sided {
        s.push_str("                double_sided: true,\n");
    }
    match node.alpha_mode {
        crate::model::SceneAlphaMode::Blend => {
            s.push_str("                alpha_mode: AlphaMode::Blend,\n");
        }
        crate::model::SceneAlphaMode::Mask => {
            s.push_str(&format!("                alpha_mode: AlphaMode::Mask({:.3}),\n", node.alpha_cutoff));
        }
        crate::model::SceneAlphaMode::AlphaToCoverage => {
            s.push_str("                alpha_mode: AlphaMode::AlphaToCoverage,\n");
        }
        crate::model::SceneAlphaMode::Opaque => {}
    }
    s.push_str("                ..default()\n            }");
    s
}

fn color_expr(c: &[f32; 4]) -> String {
    format!("Color::srgba({:.3}, {:.3}, {:.3}, {:.3})", c[0], c[1], c[2], c[3])
}

fn find_script<'a>(scripts: &'a [ScriptInfo], path: &str) -> Option<&'a ScriptInfo> {
    let stem = Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    scripts.iter().find(|s| s.mod_name == stem)
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
            }
        })
        .collect()
}
