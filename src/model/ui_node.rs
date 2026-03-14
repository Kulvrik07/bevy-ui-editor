use std::fmt;

use bevy::prelude::Resource;

// ─── Value types ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

impl Default for EditorVal {
    fn default() -> Self {
        EditorVal::Auto
    }
}

impl fmt::Display for EditorVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorVal::Auto => write!(f, "Auto"),
            EditorVal::Px(v) => write!(f, "Px({v:.1})"),
            EditorVal::Percent(v) => write!(f, "Percent({v:.1})"),
            EditorVal::Vw(v) => write!(f, "Vw({v:.1})"),
            EditorVal::Vh(v) => write!(f, "Vh({v:.1})"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorRect {
    pub top: EditorVal,
    pub right: EditorVal,
    pub bottom: EditorVal,
    pub left: EditorVal,
}

impl Default for EditorRect {
    fn default() -> Self {
        EditorRect {
            top: EditorVal::Px(0.0),
            right: EditorVal::Px(0.0),
            bottom: EditorVal::Px(0.0),
            left: EditorVal::Px(0.0),
        }
    }
}

// ─── Layout enums ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl Default for EditorFlexDirection {
    fn default() -> Self {
        EditorFlexDirection::Row
    }
}

impl fmt::Display for EditorFlexDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorFlexDirection::Row => write!(f, "Row"),
            EditorFlexDirection::Column => write!(f, "Column"),
            EditorFlexDirection::RowReverse => write!(f, "RowReverse"),
            EditorFlexDirection::ColumnReverse => write!(f, "ColumnReverse"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorJustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for EditorJustifyContent {
    fn default() -> Self {
        EditorJustifyContent::FlexStart
    }
}

impl fmt::Display for EditorJustifyContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorJustifyContent::FlexStart => write!(f, "FlexStart"),
            EditorJustifyContent::FlexEnd => write!(f, "FlexEnd"),
            EditorJustifyContent::Center => write!(f, "Center"),
            EditorJustifyContent::SpaceBetween => write!(f, "SpaceBetween"),
            EditorJustifyContent::SpaceAround => write!(f, "SpaceAround"),
            EditorJustifyContent::SpaceEvenly => write!(f, "SpaceEvenly"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorAlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

impl Default for EditorAlignItems {
    fn default() -> Self {
        EditorAlignItems::Stretch
    }
}

impl fmt::Display for EditorAlignItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorAlignItems::FlexStart => write!(f, "FlexStart"),
            EditorAlignItems::FlexEnd => write!(f, "FlexEnd"),
            EditorAlignItems::Center => write!(f, "Center"),
            EditorAlignItems::Stretch => write!(f, "Stretch"),
            EditorAlignItems::Baseline => write!(f, "Baseline"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorPositionType {
    Relative,
    Absolute,
}

impl Default for EditorPositionType {
    fn default() -> Self {
        EditorPositionType::Relative
    }
}

impl fmt::Display for EditorPositionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorPositionType::Relative => write!(f, "Relative"),
            EditorPositionType::Absolute => write!(f, "Absolute"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorOverflow {
    Visible,
    Clip,
    Hidden,
}

impl Default for EditorOverflow {
    fn default() -> Self {
        EditorOverflow::Visible
    }
}

impl fmt::Display for EditorOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorOverflow::Visible => write!(f, "Visible"),
            EditorOverflow::Clip => write!(f, "Clip"),
            EditorOverflow::Hidden => write!(f, "Hidden"),
        }
    }
}

// ─── Node type ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EditorNodeType {
    Container,
    Text,
    Button,
    Image,
}

impl Default for EditorNodeType {
    fn default() -> Self {
        EditorNodeType::Container
    }
}

impl fmt::Display for EditorNodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditorNodeType::Container => write!(f, "Container"),
            EditorNodeType::Text => write!(f, "Text"),
            EditorNodeType::Button => write!(f, "Button"),
            EditorNodeType::Image => write!(f, "Image"),
        }
    }
}

// ─── Main node struct ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct EditorUiNode {
    pub id: u64,
    pub name: String,
    pub node_type: EditorNodeType,
    pub children: Vec<EditorUiNode>,

    // Layout
    pub width: EditorVal,
    pub height: EditorVal,
    pub flex_direction: EditorFlexDirection,
    pub justify_content: EditorJustifyContent,
    pub align_items: EditorAlignItems,
    pub position_type: EditorPositionType,
    pub padding: EditorRect,
    pub margin: EditorRect,
    pub border: EditorRect,
    pub flex_wrap: bool,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: EditorVal,
    pub row_gap: EditorVal,
    pub column_gap: EditorVal,
    pub overflow_x: EditorOverflow,
    pub overflow_y: EditorOverflow,

    // Visual
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub z_index: i32,
    pub visible: bool,

    // Text / Button
    pub text_content: String,
    pub font_size: f32,
    pub text_color: [f32; 4],

    // Image
    pub image_path: String,
}

impl Default for EditorUiNode {
    fn default() -> Self {
        EditorUiNode {
            id: 0,
            name: String::from("Node"),
            node_type: EditorNodeType::Container,
            children: Vec::new(),

            width: EditorVal::Auto,
            height: EditorVal::Auto,
            flex_direction: EditorFlexDirection::Row,
            justify_content: EditorJustifyContent::FlexStart,
            align_items: EditorAlignItems::Stretch,
            position_type: EditorPositionType::Relative,
            padding: EditorRect::default(),
            margin: EditorRect::default(),
            border: EditorRect::default(),
            flex_wrap: false,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: EditorVal::Auto,
            row_gap: EditorVal::Px(0.0),
            column_gap: EditorVal::Px(0.0),
            overflow_x: EditorOverflow::Visible,
            overflow_y: EditorOverflow::Visible,

            background_color: [0.0, 0.0, 0.0, 0.0],
            border_color: [0.0, 0.0, 0.0, 0.0],
            border_radius: 0.0,
            z_index: 0,
            visible: true,

            text_content: String::new(),
            font_size: 16.0,
            text_color: [1.0, 1.0, 1.0, 1.0],

            image_path: String::new(),
        }
    }
}

// ─── Editor resources ─────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct EditorSelection {
    pub selected: Option<u64>,
}

#[derive(Resource)]
pub struct EditorIdCounter {
    counter: u64,
}

impl Default for EditorIdCounter {
    fn default() -> Self {
        EditorIdCounter { counter: 1 }
    }
}

impl EditorIdCounter {
    pub fn next_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

#[derive(Resource)]
pub struct EditorChanged {
    pub dirty: bool,
}

impl Default for EditorChanged {
    fn default() -> Self {
        EditorChanged { dirty: true }
    }
}

#[derive(Resource, Default)]
pub struct ShowExportWindow {
    pub show: bool,
    pub code: String,
}

// ─── Document resource ────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct EditorDocument {
    pub roots: Vec<EditorUiNode>,
}

impl Default for EditorDocument {
    fn default() -> Self {
        let root = EditorUiNode {
            id: 1,
            name: String::from("Root"),
            node_type: EditorNodeType::Container,
            width: EditorVal::Percent(100.0),
            height: EditorVal::Percent(100.0),
            ..Default::default()
        };
        EditorDocument { roots: vec![root] }
    }
}

impl EditorDocument {
    pub fn add_child(&mut self, parent_id: Option<u64>, node: EditorUiNode) {
        match parent_id {
            None => self.roots.push(node),
            Some(pid) => {
                if let Some(parent) = self.find_node_mut(pid) {
                    parent.children.push(node);
                }
            }
        }
    }

    pub fn remove_node(&mut self, id: u64) -> bool {
        fn remove_from(nodes: &mut Vec<EditorUiNode>, id: u64) -> bool {
            if let Some(pos) = nodes.iter().position(|n| n.id == id) {
                nodes.remove(pos);
                return true;
            }
            for node in nodes.iter_mut() {
                if remove_from(&mut node.children, id) {
                    return true;
                }
            }
            false
        }
        remove_from(&mut self.roots, id)
    }

    pub fn find_node(&self, id: u64) -> Option<&EditorUiNode> {
        fn find_in(nodes: &[EditorUiNode], id: u64) -> Option<&EditorUiNode> {
            for node in nodes {
                if node.id == id {
                    return Some(node);
                }
                if let Some(found) = find_in(&node.children, id) {
                    return Some(found);
                }
            }
            None
        }
        find_in(&self.roots, id)
    }

    pub fn find_node_mut(&mut self, id: u64) -> Option<&mut EditorUiNode> {
        fn find_in_mut(nodes: &mut Vec<EditorUiNode>, id: u64) -> Option<&mut EditorUiNode> {
            for node in nodes.iter_mut() {
                if node.id == id {
                    return Some(node);
                }
                if let Some(found) = find_in_mut(&mut node.children, id) {
                    return Some(found);
                }
            }
            None
        }
        find_in_mut(&mut self.roots, id)
    }
}
