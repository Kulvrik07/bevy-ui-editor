# Bevy UI Editor

A visual UI editor for designing **Bevy game engine** UI layouts. Instead of writing Bevy UI code by hand, use the graphical interface to drag, configure, and arrange nodes — then export clean, idiomatic Bevy 0.18 Rust source code.

The editor is built **in Bevy itself** using **`bevy_egui`** for the editor panels, with a live Bevy viewport showing a real-time preview of the UI being designed.

<!-- screenshot -->

## Features

- **Hierarchy panel** — tree view of UI nodes with add/delete/select actions
- **Inspector panel** — edit all node properties: layout, spacing, colors, text, visibility
- **Live viewport** — real-time Bevy UI preview in the center, updates as you edit
- **Toolbar** — quick access to add nodes, delete selected, and export
- **Rust code export** — generates clean, valid Bevy 0.18 Rust source code with copy-to-clipboard

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

