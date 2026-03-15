use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Sense, Stroke, Vec2};
use bevy_egui::EguiContexts;

use crate::model::{ConsoleLog, EditorState, SceneChanged, SceneDocument, SceneIdCounter, SceneNodeKind, SceneSelection, ScriptRef, UndoHistory, new_scene_node};

// ─── Resource ─────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct FileExplorerState {
    pub root: PathBuf,
    /// Folder tree entries (left pane)
    pub folder_tree: Vec<FolderEntry>,
    /// Currently viewed directory (right pane shows its contents)
    pub current_dir: PathBuf,
    /// Cached contents of current_dir
    pub dir_contents: Vec<ContentEntry>,
    pub needs_refresh: bool,
    pub needs_content_refresh: bool,
    pub rename_target: Option<PathBuf>,
    pub rename_buf: String,
    pub new_folder_parent: Option<PathBuf>,
    pub new_folder_name: String,
    /// Selected file in right pane
    pub selected_file: Option<PathBuf>,
}

impl Default for FileExplorerState {
    fn default() -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        FileExplorerState {
            current_dir: root.clone(),
            root,
            folder_tree: Vec::new(),
            dir_contents: Vec::new(),
            needs_refresh: true,
            needs_content_refresh: true,
            rename_target: None,
            rename_buf: String::new(),
            new_folder_parent: None,
            new_folder_name: String::new(),
            selected_file: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FolderEntry {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub expanded: bool,
    pub has_subdirs: bool,
}

#[derive(Clone, Debug)]
pub struct ContentEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

// ─── Scan helpers ─────────────────────────────────────────────────────────────

fn is_hidden(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

fn scan_folder_tree(dir: &Path, depth: usize, max_depth: usize) -> Vec<FolderEntry> {
    if depth > max_depth {
        return Vec::new();
    }
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut folders: Vec<_> = read
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !is_hidden(&name) && e.file_type().map(|f| f.is_dir()).unwrap_or(false)
        })
        .collect();
    folders.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let mut result = Vec::new();
    for entry in folders {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let has_subdirs = std::fs::read_dir(&path)
            .map(|rd| {
                rd.filter_map(|e| e.ok()).any(|e| {
                    let n = e.file_name().to_string_lossy().to_string();
                    !is_hidden(&n) && e.file_type().map(|f| f.is_dir()).unwrap_or(false)
                })
            })
            .unwrap_or(false);
        result.push(FolderEntry {
            path,
            name,
            depth,
            expanded: false,
            has_subdirs,
        });
    }
    result
}

fn scan_dir_contents(dir: &Path) -> Vec<ContentEntry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<ContentEntry> = read
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !is_hidden(&name)
        })
        .map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let is_dir = e.file_type().map(|f| f.is_dir()).unwrap_or(false);
            let size_bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
            ContentEntry {
                path: e.path(),
                name,
                is_dir,
                size_bytes,
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name))
    });
    entries
}

// ─── Icons ────────────────────────────────────────────────────────────────────

fn file_icon_large(name: &str, is_dir: bool) -> (&'static str, Color32) {
    if is_dir {
        return ("📂", Color32::from_rgb(230, 190, 70));
    }
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => ("🦀", Color32::from_rgb(220, 120, 60)),
        "toml" | "yaml" | "yml" => ("⚙", Color32::from_rgb(160, 160, 170)),
        "json" => ("{ }", Color32::from_rgb(130, 200, 110)),
        "md" | "txt" => ("📝", Color32::from_rgb(120, 170, 230)),
        "png" | "jpg" | "jpeg" | "bmp" | "webp" => ("🖼", Color32::from_rgb(190, 130, 210)),
        "svg" => ("◇", Color32::from_rgb(255, 180, 70)),
        "glb" | "gltf" | "obj" | "fbx" => ("🎨", Color32::from_rgb(100, 210, 190)),
        "wav" | "ogg" | "mp3" | "flac" => ("♪", Color32::from_rgb(80, 200, 220)),
        "wasm" => ("⬡", Color32::from_rgb(100, 80, 200)),
        "lock" => ("🔒", Color32::from_rgb(100, 100, 100)),
        "sh" | "bat" | "cmd" => (">_", Color32::from_rgb(100, 200, 100)),
        "html" | "css" | "js" | "ts" => ("</>", Color32::from_rgb(230, 160, 60)),
        _ => ("📄", Color32::from_rgb(170, 170, 170)),
    }
}

fn file_size_string(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ─── Colors ───────────────────────────────────────────────────────────────────

const BG_DARK: Color32 = Color32::from_rgb(25, 25, 25);
const BG_PANEL: Color32 = Color32::from_rgb(35, 35, 35);
const BG_HOVER: Color32 = Color32::from_rgb(50, 55, 65);
const BG_SELECTED: Color32 = Color32::from_rgb(40, 65, 110);
const SEPARATOR: Color32 = Color32::from_rgb(55, 55, 55);
const TEXT_DIM: Color32 = Color32::from_rgb(120, 120, 130);
const TEXT_NORMAL: Color32 = Color32::from_rgb(200, 200, 205);
const TEXT_BRIGHT: Color32 = Color32::from_rgb(235, 235, 240);
const BREADCRUMB_SEP: Color32 = Color32::from_rgb(80, 80, 90);

// ─── System ───────────────────────────────────────────────────────────────────

pub fn file_explorer_system(
    mut ctx: EguiContexts,
    mut state: ResMut<FileExplorerState>,
    mut doc: ResMut<SceneDocument>,
    mut editor: ResMut<EditorState>,
    mut changed: ResMut<SceneChanged>,
    mut undo: ResMut<UndoHistory>,
    mut console: ResMut<ConsoleLog>,
    mut id_counter: ResMut<SceneIdCounter>,
    selection: Res<SceneSelection>,
    app_mode: Res<super::launcher::AppModeRes>,
) {
    if app_mode.mode != super::launcher::AppMode::Editor { return; }
    if editor.play_mode { return; }
    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    // Refresh folder tree
    if state.needs_refresh {
        state.folder_tree = build_folder_tree(&state.root.clone());
        state.needs_refresh = false;
        state.needs_content_refresh = true;
    }
    // Refresh current directory content
    if state.needs_content_refresh {
        state.dir_contents = scan_dir_contents(&state.current_dir.clone());
        state.needs_content_refresh = false;
    }

    let mut load_file: Option<PathBuf> = None;
    let mut action: Option<ExplorerAction> = None;
    let mut navigate_to: Option<PathBuf> = None;

    egui::TopBottomPanel::bottom("file_explorer")
        .default_height(220.0)
        .height_range(60.0..=500.0)
        .resizable(true)
        .show_separator_line(true)
        .frame(Frame {
            fill: BG_DARK,
            inner_margin: Margin::symmetric(0, 2),
            ..Default::default()
        })
        .show(egui_ctx, |ui| {
            // Claim full available height so the panel stores the correct
            // resized rect and doesn't snap back to content size.
            ui.set_min_height(ui.available_height());

            // ── Toolbar with breadcrumbs ──────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.label(RichText::new("📁 Project").strong().color(TEXT_BRIGHT).size(12.0));

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Breadcrumb navigation
                let root = state.root.clone();
                let current = state.current_dir.clone();
                if let Ok(rel) = current.strip_prefix(&root) {
                    let root_name = root.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Project".to_string());
                    if ui.add(egui::Label::new(
                        RichText::new(&root_name).color(TEXT_NORMAL).size(11.0)
                    ).sense(Sense::click())).clicked() {
                        navigate_to = Some(root.clone());
                    }

                    let mut accumulated = root.clone();
                    for component in rel.components() {
                        let part = component.as_os_str().to_string_lossy().to_string();
                        accumulated = accumulated.join(&part);
                        ui.label(RichText::new("›").color(BREADCRUMB_SEP).size(11.0));
                        let target = accumulated.clone();
                        if ui.add(egui::Label::new(
                            RichText::new(&part).color(TEXT_NORMAL).size(11.0)
                        ).sense(Sense::click())).clicked() {
                            if target.is_dir() {
                                navigate_to = Some(target);
                            }
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(6.0);
                    if ui.add(egui::Button::new(
                        RichText::new("🔄").size(12.0)
                    ).fill(Color32::TRANSPARENT)).on_hover_text("Refresh").clicked() {
                        state.needs_refresh = true;
                    }
                });
            });

            // Thin separator line
            let sep_rect = ui.allocate_space(egui::Vec2::new(ui.available_width(), 1.0));
            ui.painter().rect_filled(sep_rect.1, 0.0, SEPARATOR);

            // ── Two-pane layout ──────────────────────────────────────
            let available = ui.available_rect_before_wrap();
            let pane_height = available.height();
            let tree_width = 180.0_f32.min(available.width() * 0.3);

            ui.horizontal(|ui| {
                // ── LEFT: Folder tree ────────────────────────────────
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(tree_width, pane_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_min_width(tree_width);
                        let bg_rect = ui.available_rect_before_wrap();
                        ui.painter().rect_filled(bg_rect, 0.0, BG_PANEL);

                        ScrollArea::vertical().id_salt("folder_tree_scroll").show(ui, |ui| {
                            ui.add_space(4.0);
                            let root_path = state.root.clone();
                            let root_name = root_path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Project".to_string());
                            let current_dir = state.current_dir.clone();

                            // Root folder
                            let is_current = current_dir == root_path;
                            let root_bg = if is_current { BG_SELECTED } else { Color32::TRANSPARENT };
                            let resp = ui.horizontal(|ui| {
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 20.0),
                                    Sense::click(),
                                );
                                if resp.hovered() || is_current {
                                    ui.painter().rect_filled(rect, 2.0, if resp.hovered() && !is_current { BG_HOVER } else { root_bg });
                                }
                                let text_pos = rect.left_center() + egui::Vec2::new(6.0, 0.0);
                                ui.painter().text(
                                    text_pos,
                                    egui::Align2::LEFT_CENTER,
                                    format!("📁 {root_name}"),
                                    egui::FontId::proportional(11.0),
                                    if is_current { TEXT_BRIGHT } else { TEXT_NORMAL },
                                );
                                resp
                            });
                            if resp.inner.clicked() {
                                navigate_to = Some(root_path.clone());
                            }

                            // Folder tree entries
                            let tree_clone = state.folder_tree.clone();
                            let mut toggle_idx: Option<usize> = None;
                            for (idx, folder) in tree_clone.iter().enumerate() {
                                let is_current = current_dir == folder.path;
                                let indent = (folder.depth + 1) as f32 * 14.0;
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 20.0),
                                    Sense::click(),
                                );
                                let bg = if is_current {
                                    BG_SELECTED
                                } else if resp.hovered() {
                                    BG_HOVER
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(rect, 2.0, bg);

                                // Arrow for expandable folders
                                if folder.has_subdirs {
                                    let arrow = if folder.expanded { "▾" } else { "▸" };
                                    let arrow_pos = rect.left_center() + egui::Vec2::new(indent - 10.0, 0.0);
                                    ui.painter().text(
                                        arrow_pos,
                                        egui::Align2::LEFT_CENTER,
                                        arrow,
                                        egui::FontId::proportional(10.0),
                                        TEXT_DIM,
                                    );
                                }

                                let icon = if folder.expanded && folder.has_subdirs { "📂" } else { "📁" };
                                let text_pos = rect.left_center() + egui::Vec2::new(indent + 2.0, 0.0);
                                ui.painter().text(
                                    text_pos,
                                    egui::Align2::LEFT_CENTER,
                                    format!("{icon} {}", folder.name),
                                    egui::FontId::proportional(11.0),
                                    if is_current { TEXT_BRIGHT } else { TEXT_NORMAL },
                                );

                                if resp.clicked() {
                                    navigate_to = Some(folder.path.clone());
                                    if folder.has_subdirs && !folder.expanded {
                                        toggle_idx = Some(idx);
                                    }
                                }
                                if resp.double_clicked() && folder.has_subdirs {
                                    toggle_idx = Some(idx);
                                }

                                // Context menu on folders
                                resp.context_menu(|ui| {
                                    if ui.button("📁 New Folder").clicked() {
                                        action = Some(ExplorerAction::NewFolder(folder.path.clone()));
                                        ui.close();
                                    }
                                    if ui.button("📄 New Scene File").clicked() {
                                        action = Some(ExplorerAction::NewSceneFile(folder.path.clone()));
                                        ui.close();
                                    }
                                    if ui.button("📜 New Script").clicked() {
                                        action = Some(ExplorerAction::NewScript(folder.path.clone()));
                                        ui.close();
                                    }
                                });
                            }

                            // Toggle folder expansion
                            if let Some(idx) = toggle_idx {
                                let folder = &state.folder_tree[idx];
                                let was_expanded = folder.expanded;
                                let folder_path = folder.path.clone();
                                let depth = folder.depth;

                                if was_expanded {
                                    state.folder_tree[idx].expanded = false;
                                    let remove_pos = idx + 1;
                                    while state.folder_tree.len() > remove_pos
                                        && state.folder_tree[remove_pos].depth > depth
                                    {
                                        state.folder_tree.remove(remove_pos);
                                    }
                                } else {
                                    state.folder_tree[idx].expanded = true;
                                    let children = scan_folder_tree(&folder_path, depth + 1, depth + 1);
                                    for (i, child) in children.into_iter().enumerate() {
                                        state.folder_tree.insert(idx + 1 + i, child);
                                    }
                                }
                            }

                            ui.add_space(4.0);
                        });
                    },
                );

                // Vertical separator
                let sep_x = ui.cursor().left();
                let top = available.top();
                let bot = available.bottom();
                ui.painter().line_segment(
                    [egui::Pos2::new(sep_x, top), egui::Pos2::new(sep_x, bot)],
                    Stroke::new(1.0, SEPARATOR),
                );

                // ── RIGHT: File content grid ─────────────────────────
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), pane_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                    let right_bg = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(right_bg, 0.0, BG_DARK);

                    // New folder form
                    if state.new_folder_parent.is_some() {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(RichText::new("📁 New folder:").color(TEXT_NORMAL).size(11.0));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut state.new_folder_name)
                                    .desired_width(200.0)
                                    .hint_text("folder name"),
                            );
                            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                let name = state.new_folder_name.trim().to_string();
                                if !name.is_empty() {
                                    if let Some(parent) = &state.new_folder_parent {
                                        let new_path = parent.join(&name);
                                        match std::fs::create_dir_all(&new_path) {
                                            Ok(()) => {
                                                console.info(format!("Created folder: {}", new_path.display()));
                                                state.needs_refresh = true;
                                            }
                                            Err(e) => console.error(format!("Failed to create folder: {e}")),
                                        }
                                    }
                                }
                                state.new_folder_parent = None;
                                state.new_folder_name.clear();
                            }
                            if ui.small_button("✕").clicked() {
                                state.new_folder_parent = None;
                                state.new_folder_name.clear();
                            }
                        });
                    }

                    ScrollArea::vertical().id_salt("content_scroll").show(ui, |ui| {
                        ui.add_space(4.0);
                        let content_width = ui.available_width();
                        let item_width = 90.0_f32;
                        let item_height = 92.0_f32;
                        let spacing = 6.0;
                        let cols = ((content_width - 8.0) / (item_width + spacing)).max(1.0) as usize;

                        let contents_clone = state.dir_contents.clone();
                        let selected = state.selected_file.clone();
                        let rename_target = state.rename_target.clone();

                        let mut rows = contents_clone.chunks(cols).peekable();
                        while let Some(row) = rows.next() {
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                for entry in row {
                                    let is_sel = selected.as_ref() == Some(&entry.path);
                                    let is_renaming = rename_target.as_ref() == Some(&entry.path);

                                    let (rect, resp) = ui.allocate_exact_size(
                                        Vec2::new(item_width, item_height),
                                        Sense::click(),
                                    );

                                    // Background
                                    let bg = if is_sel {
                                        BG_SELECTED
                                    } else if resp.hovered() {
                                        BG_HOVER
                                    } else {
                                        Color32::TRANSPARENT
                                    };
                                    ui.painter().rect_filled(rect, CornerRadius::same(4), bg);

                                    // Icon
                                    let (icon, icon_color) = file_icon_large(&entry.name, entry.is_dir);
                                    let icon_center = egui::Pos2::new(
                                        rect.center().x,
                                        rect.top() + 30.0,
                                    );
                                    ui.painter().text(
                                        icon_center,
                                        egui::Align2::CENTER_CENTER,
                                        icon,
                                        egui::FontId::proportional(24.0),
                                        icon_color,
                                    );

                                    // File name
                                    if is_renaming {
                                        let text_rect = egui::Rect::from_min_size(
                                            egui::Pos2::new(rect.left() + 2.0, rect.top() + 54.0),
                                            Vec2::new(item_width - 4.0, 32.0),
                                        );
                                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(text_rect));
                                        let resp = child.add(
                                            egui::TextEdit::singleline(&mut state.rename_buf)
                                                .desired_width(item_width - 8.0)
                                                .font(egui::TextStyle::Small),
                                        );
                                        if resp.lost_focus() {
                                            let new_name = state.rename_buf.trim().to_string();
                                            if !new_name.is_empty() && new_name != entry.name {
                                                let new_path = entry.path.parent().unwrap_or(Path::new(".")).join(&new_name);
                                                match std::fs::rename(&entry.path, &new_path) {
                                                    Ok(()) => {
                                                        console.info(format!("Renamed to {new_name}"));
                                                        state.needs_refresh = true;
                                                    }
                                                    Err(e) => console.error(format!("Rename failed: {e}")),
                                                }
                                            }
                                            state.rename_target = None;
                                            state.rename_buf.clear();
                                        }
                                        resp.request_focus();
                                    } else {
                                        let display_name = if entry.name.len() > 12 {
                                            format!("{}…", &entry.name[..11])
                                        } else {
                                            entry.name.clone()
                                        };
                                        let name_color = if entry.is_dir {
                                            Color32::from_rgb(230, 190, 70)
                                        } else if entry.name.ends_with(".json") {
                                            Color32::from_rgb(130, 200, 130)
                                        } else {
                                            TEXT_NORMAL
                                        };
                                        let name_pos = egui::Pos2::new(
                                            rect.center().x,
                                            rect.top() + 60.0,
                                        );
                                        ui.painter().text(
                                            name_pos,
                                            egui::Align2::CENTER_TOP,
                                            &display_name,
                                            egui::FontId::proportional(11.0),
                                            name_color,
                                        );

                                        if !entry.is_dir {
                                            ui.painter().text(
                                                egui::Pos2::new(rect.center().x, rect.bottom() - 4.0),
                                                egui::Align2::CENTER_BOTTOM,
                                                file_size_string(entry.size_bytes),
                                                egui::FontId::proportional(8.0),
                                                TEXT_DIM,
                                            );
                                        }
                                    }

                                    // Click: select
                                    if resp.clicked() {
                                        state.selected_file = Some(entry.path.clone());
                                    }

                                    // Double-click: navigate into dir or load json
                                    if resp.double_clicked() {
                                        if entry.is_dir {
                                            navigate_to = Some(entry.path.clone());
                                        } else if entry.name.ends_with(".json") {
                                            load_file = Some(entry.path.clone());
                                        }
                                    }

                                    // Context menu
                                    resp.context_menu(|ui| {
                                        if entry.is_dir {
                                            if ui.button("📂 Open Folder").clicked() {
                                                navigate_to = Some(entry.path.clone());
                                                ui.close();
                                            }
                                            if ui.button("📁 New Subfolder").clicked() {
                                                action = Some(ExplorerAction::NewFolder(entry.path.clone()));
                                                ui.close();
                                            }
                                            ui.separator();
                                        }
                                        if entry.name.ends_with(".json") {
                                            if ui.button("📂 Load Scene").clicked() {
                                                load_file = Some(entry.path.clone());
                                                ui.close();
                                            }
                                            ui.separator();
                                        }
                                        // 3D model import
                                        let ext = entry.name.rsplit('.').next().unwrap_or("").to_lowercase();
                                        if matches!(ext.as_str(), "glb" | "gltf" | "fbx" | "obj") {
                                            if ui.button("🎨 Import as 3D Model").clicked() {
                                                action = Some(ExplorerAction::ImportModel(entry.path.clone()));
                                                ui.close();
                                            }
                                            ui.separator();
                                        }
                                        // Script attachment
                                        if matches!(ext.as_str(), "rs" | "lua" | "rhai" | "py" | "js" | "ts" | "wasm") {
                                            if ui.button("📜 Attach as Script").clicked() {
                                                action = Some(ExplorerAction::ImportScript(entry.path.clone()));
                                                ui.close();
                                            }
                                            ui.separator();
                                        }
                                        // Copy to assets
                                        if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "webp" | "hdr" | "wav" | "ogg" | "mp3" | "flac" | "glb" | "gltf" | "fbx" | "obj" | "svg") {
                                            if ui.button("📦 Copy to Assets").clicked() {
                                                action = Some(ExplorerAction::CopyToAssets(entry.path.clone()));
                                                ui.close();
                                            }
                                        }
                                        if ui.button("✏  Rename").clicked() {
                                            action = Some(ExplorerAction::Rename(entry.path.clone(), entry.name.clone()));
                                            ui.close();
                                        }
                                        if ui.button("🗑  Delete").clicked() {
                                            action = Some(ExplorerAction::Delete(entry.path.clone(), entry.is_dir));
                                            ui.close();
                                        }
                                        ui.separator();
                                        if ui.button("📋 Copy Path").clicked() {
                                            ui.ctx().copy_text(entry.path.display().to_string());
                                            ui.close();
                                        }
                                    });

                                    ui.add_space(spacing);
                                }
                            });
                            ui.add_space(spacing);
                        }

                        // Right-click on empty space
                        let remaining = ui.allocate_response(ui.available_size(), Sense::click());
                        remaining.context_menu(|ui| {
                            let cd = state.current_dir.clone();
                            if ui.button("📁 New Folder").clicked() {
                                action = Some(ExplorerAction::NewFolder(cd.clone()));
                                ui.close();
                            }
                            if ui.button("📄 New Scene File").clicked() {
                                action = Some(ExplorerAction::NewSceneFile(cd.clone()));
                                ui.close();
                            }
                            if ui.button("📜 New Script").clicked() {
                                action = Some(ExplorerAction::NewScript(cd));
                                ui.close();
                            }
                            if ui.button("🔄 Refresh").clicked() {
                                state.needs_refresh = true;
                                ui.close();
                            }
                        });
                    });
                });
            });
        });

    // ── Navigate to directory ─────────────────────────────────────────
    if let Some(path) = navigate_to {
        if path.is_dir() {
            state.current_dir = path;
            state.needs_content_refresh = true;
        }
    }

    // ── Process actions ───────────────────────────────────────────────
    if let Some(act) = action {
        match act {
            ExplorerAction::NewFolder(parent) => {
                state.new_folder_parent = Some(parent);
                state.new_folder_name.clear();
            }
            ExplorerAction::NewSceneFile(parent) => {
                let file_path = parent.join("new_scene.json");
                let default_json = "{\n  \"nodes\": []\n}";
                match std::fs::write(&file_path, default_json) {
                    Ok(()) => {
                        console.info(format!("Created scene: {}", file_path.display()));
                        state.needs_refresh = true;
                    }
                    Err(e) => console.error(format!("Failed to create file: {e}")),
                }
            }
            ExplorerAction::NewScript(parent) => {
                // Create a scripts/ dir if needed, then create template
                let scripts_dir = if parent.ends_with("scripts") {
                    parent.clone()
                } else {
                    parent.join("scripts")
                };
                let _ = std::fs::create_dir_all(&scripts_dir);

                // Find a unique name
                let mut name = "new_script.rs".to_string();
                let mut counter = 1u32;
                while scripts_dir.join(&name).exists() {
                    counter += 1;
                    name = format!("new_script_{counter}.rs");
                }
                let file_path = scripts_dir.join(&name);
                let pascal = to_pascal_case(&name.replace(".rs", ""));
                let snake = name.replace(".rs", "");
                let template = format!(
                    "use bevy::prelude::*;\n\
                     \n\
                     /// Marker component — attached automatically to the entity in the scene.\n\
                     #[derive(Component)]\n\
                     pub struct {pascal};\n\
                     \n\
                     /// Runs every frame for each entity that has the `{pascal}` component.\n\
                     pub fn {snake}_update(\n\
                         time: Res<Time>,\n\
                         mut query: Query<&mut Transform, With<{pascal}>>,\n\
                     ) {{\n\
                         for mut transform in &mut query {{\n\
                             // Example: rotate around Y axis\n\
                             transform.rotate_y(1.0 * time.delta_secs());\n\
                         }}\n\
                     }}\n",
                );
                match std::fs::write(&file_path, template) {
                    Ok(()) => {
                        console.info(format!("Created script: {}", file_path.display()));
                        state.needs_refresh = true;
                        state.needs_content_refresh = true;
                    }
                    Err(e) => console.error(format!("Failed to create script: {e}")),
                }
            }
            ExplorerAction::Rename(path, name) => {
                state.rename_target = Some(path);
                state.rename_buf = name;
            }
            ExplorerAction::Delete(path, is_dir) => {
                let result = if is_dir {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                match result {
                    Ok(()) => {
                        console.info(format!("Deleted: {}", path.display()));
                        state.needs_refresh = true;
                    }
                    Err(e) => console.error(format!("Delete failed: {e}")),
                }
            }
            ExplorerAction::ImportModel(path) => {
                // Copy to assets/models/ and add a Model node to the scene
                let assets_dir = state.root.join("assets").join("models");
                let _ = std::fs::create_dir_all(&assets_dir);
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let dest = assets_dir.join(&file_name);
                if path != dest {
                    match std::fs::copy(&path, &dest) {
                        Ok(_) => console.info(format!("Copied to assets/models/{file_name}")),
                        Err(e) => {
                            console.error(format!("Failed to copy model: {e}"));
                            return;
                        }
                    }
                }
                let asset_path = format!("models/{file_name}");
                let id = id_counter.next_id();
                let node = new_scene_node(id, SceneNodeKind::Model(asset_path));
                undo.push_snapshot(&doc.nodes);
                doc.add_node(None, node);
                changed.dirty = true;
                console.info(format!("Imported model: {file_name}"));
                state.needs_refresh = true;
            }
            ExplorerAction::ImportScript(path) => {
                // Attach script to the currently selected node
                let rel_path = path.strip_prefix(&state.root)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string_lossy().to_string());
                if let Some(sel_id) = selection.selected {
                    if let Some(node) = doc.find_node_mut(sel_id) {
                        if !node.scripts.iter().any(|s| s.path == rel_path) {
                            node.scripts.push(ScriptRef {
                                path: rel_path.clone(),
                                enabled: true,
                            });
                            changed.dirty = true;
                            console.info(format!("Attached script '{}' to '{}'", rel_path, node.name));
                        } else {
                            console.warn(format!("Script '{}' already attached", rel_path));
                        }
                    }
                } else {
                    console.warn(format!("Select a node first, then attach script: {rel_path}"));
                }
            }
            ExplorerAction::CopyToAssets(path) => {
                let ext = path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
                let subdir = match ext.as_str() {
                    "glb" | "gltf" | "fbx" | "obj" => "models",
                    "png" | "jpg" | "jpeg" | "bmp" | "webp" | "hdr" | "svg" => "textures",
                    "wav" | "ogg" | "mp3" | "flac" => "audio",
                    _ => "misc",
                };
                let assets_dir = state.root.join("assets").join(subdir);
                let _ = std::fs::create_dir_all(&assets_dir);
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let dest = assets_dir.join(&file_name);
                match std::fs::copy(&path, &dest) {
                    Ok(_) => {
                        console.info(format!("Copied to assets/{subdir}/{file_name}"));
                        state.needs_refresh = true;
                    }
                    Err(e) => console.error(format!("Copy failed: {e}")),
                }
            }
        }
    }

    // ── Load scene file ───────────────────────────────────────────────
    if let Some(path) = load_file {
        match std::fs::read_to_string(&path) {
            Ok(json) => match SceneDocument::from_json(&json) {
                Ok(new_doc) => {
                    undo.push_snapshot(&doc.nodes);
                    *doc = new_doc;
                    editor.scene_file_path = Some(path.to_string_lossy().to_string());
                    editor.scene_dirty = false;
                    changed.dirty = true;
                    console.info(format!("Loaded scene: {}", path.display()));
                }
                Err(e) => {
                    console.error(format!("Failed to parse scene: {e}"));
                }
            },
            Err(e) => {
                console.error(format!("Failed to read file: {e}"));
            }
        }
    }
}

// ─── Build initial folder tree ────────────────────────────────────────────────

fn build_folder_tree(root: &Path) -> Vec<FolderEntry> {
    let mut tree = scan_folder_tree(root, 0, 0);
    // Auto-expand first level
    let mut expanded = Vec::new();
    for (i, entry) in tree.iter_mut().enumerate() {
        if entry.has_subdirs {
            entry.expanded = true;
            expanded.push(i);
        }
    }
    // Insert children for expanded first-level folders (reverse to keep indices valid)
    for &idx in expanded.iter().rev() {
        let children = scan_folder_tree(&tree[idx].path, 1, 1);
        for (j, child) in children.into_iter().enumerate() {
            tree.insert(idx + 1 + j, child);
        }
    }
    tree
}

// ─── Actions enum ─────────────────────────────────────────────────────────────

enum ExplorerAction {
    NewFolder(PathBuf),
    NewSceneFile(PathBuf),
    NewScript(PathBuf),
    Rename(PathBuf, String),
    Delete(PathBuf, bool),
    ImportModel(PathBuf),
    ImportScript(PathBuf),
    CopyToAssets(PathBuf),
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect()
}
