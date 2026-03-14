# 🎨 Bevy UI Editor

A visual UI editor for designing **Bevy game engine** UI layouts. Instead of writing Bevy UI code by hand, use the graphical interface to drag, configure, and arrange nodes — then export clean, idiomatic Bevy 0.18 Rust source code.

The editor is built **in Bevy itself** using **`bevy_egui`** for the editor panels, with a live Bevy viewport showing a real-time preview of the UI being designed.

<!-- screenshot -->

## ✨ Features

- 🌳 **Hierarchy panel** — tree view of UI nodes with add/delete/select actions
- 🔧 **Inspector panel** — edit all node properties: layout, spacing, colors, text, visibility
- 🖥️ **Live viewport** — real-time Bevy UI preview in the center, updates as you edit
- 🔨 **Toolbar** — quick access to add nodes, delete selected, and export
- 📤 **Rust code export** — generates clean, valid Bevy 0.18 Rust source code with copy-to-clipboard

### Supported Node Types

| Type | Description |
|---|---|
| **Container** | Empty flexbox node — for layout |
| **Text** | Text node with font size and color |
| **Button** | Interactive button with label |
| **Image** | Image placeholder (MVP) |

### Style Properties

- **Layout**: Width, Height (Auto/Px/Percent/Vw/Vh), Flex Direction, Justify Content, Align Items, Position Type, Flex Wrap, Flex Grow/Shrink/Basis, Row/Column Gap, Overflow
- **Spacing**: Padding, Margin, Border — each with 4 independent sides
- **Visual**: Background Color, Border Color, Border Radius, Z-Index, Visibility
- **Text**: Content, Font Size (8–120), Text Color

## 🚀 Prerequisites

- **Rust 1.80+** — install via [rustup](https://rustup.rs/)
- **cargo** — bundled with Rust
- Linux system libraries (for Bevy):
  ```bash
  # Ubuntu/Debian
  sudo apt-get install -y libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
  ```

## ⚡ Quick Start

```bash
git clone https://github.com/Kulvrik07/bevy-ui-editor
cd bevy-ui-editor
cargo run
```

> 💡 The first build takes a few minutes — Bevy is a large dependency. Subsequent builds are fast.

## 🎮 Usage Guide

### Adding Nodes
- Click **"+ Container"**, **"+ Text"**, **"+ Button"**, or **"+ Image"** in the toolbar or hierarchy panel
- New nodes are added as children of the currently selected node (or as root nodes if nothing is selected)

### Editing Properties
1. Click a node in the **Hierarchy** panel (left side) to select it
2. The **Inspector** panel (right side) shows all editable properties
3. Changes take effect immediately in the **live viewport** (center)

### Exporting Code
1. Click **"📤 Export Rust Code"** in the toolbar
2. A window appears with the generated Rust code
3. Click **"📋 Copy to Clipboard"** to copy it
4. Paste into your Bevy project and call `spawn_ui(commands)` in a startup system

### Example Exported Code

```rust
use bevy::prelude::*;

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Hello, Bevy!"),
                TextFont { font_size: 32.0, ..default() },
                TextColor(Color::srgba(1.000, 1.000, 1.000, 1.000)),
            ));
        });
}
```

## 📁 Project Structure

```
bevy-ui-editor/
├── src/
│   ├── main.rs                # App entry point — sets up Bevy with editor plugins
│   ├── editor/
│   │   ├── mod.rs             # EditorPlugin that registers all editor systems
│   │   ├── hierarchy.rs       # Left panel: tree view of UI node hierarchy
│   │   ├── inspector.rs       # Right panel: property editor for selected node
│   │   ├── viewport.rs        # Center: live Bevy UI preview rendering
│   │   └── toolbar.rs         # Top bar: add node, delete, export actions
│   ├── model/
│   │   ├── mod.rs             # Data model module
│   │   └── ui_node.rs         # EditorUiNode — editor representation of a Bevy UI node
│   └── export/
│       ├── mod.rs             # Export module
│       └── rust_codegen.rs    # Generates valid Bevy 0.18 Rust code from the model
└── Cargo.toml                 # bevy 0.18.1, bevy_egui 0.39.1
```

## 🗺️ Roadmap

### Phase 2
- [ ] Undo/redo system
- [ ] Drag-and-drop node reordering in the hierarchy
- [ ] Save/load project files (`.ron` or `.json`)
- [ ] Component templates (button, card, modal presets)

### Phase 3
- [ ] Animation keyframe editor
- [ ] Theming and reusable style system
- [ ] Multi-resolution preview
- [ ] Plugin system for custom components
- [ ] Image/font asset browser

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes
4. Open a pull request

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

