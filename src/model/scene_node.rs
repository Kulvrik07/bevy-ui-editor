use std::fmt;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

// ─── 3D Primitive shapes ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScenePrimitive {
    Cube,
    Sphere,
    Cylinder,
    Capsule,
    Plane,
    Torus,
    Cone,
    Tetrahedron,
}

impl Default for ScenePrimitive {
    fn default() -> Self {
        ScenePrimitive::Cube
    }
}

impl fmt::Display for ScenePrimitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenePrimitive::Cube => write!(f, "Cube"),
            ScenePrimitive::Sphere => write!(f, "Sphere"),
            ScenePrimitive::Cylinder => write!(f, "Cylinder"),
            ScenePrimitive::Capsule => write!(f, "Capsule"),
            ScenePrimitive::Plane => write!(f, "Plane"),
            ScenePrimitive::Torus => write!(f, "Torus"),
            ScenePrimitive::Cone => write!(f, "Cone"),
            ScenePrimitive::Tetrahedron => write!(f, "Tetrahedron"),
        }
    }
}

// ─── Light types ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SceneLightKind {
    Point,
    Directional,
    Spot,
}

impl Default for SceneLightKind {
    fn default() -> Self {
        SceneLightKind::Point
    }
}

impl fmt::Display for SceneLightKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SceneLightKind::Point => write!(f, "Point"),
            SceneLightKind::Directional => write!(f, "Directional"),
            SceneLightKind::Spot => write!(f, "Spot"),
        }
    }
}

// ─── Node kind ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SceneNodeKind {
    Empty,
    Mesh(ScenePrimitive),
    Light(SceneLightKind),
}

impl Default for SceneNodeKind {
    fn default() -> Self {
        SceneNodeKind::Empty
    }
}

impl fmt::Display for SceneNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SceneNodeKind::Empty => write!(f, "Empty"),
            SceneNodeKind::Mesh(p) => write!(f, "Mesh ({p})"),
            SceneNodeKind::Light(l) => write!(f, "Light ({l})"),
        }
    }
}

// ─── Scene node ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: u64,
    pub name: String,
    pub kind: SceneNodeKind,
    pub children: Vec<SceneNode>,

    pub translation: [f32; 3],
    pub rotation_euler: [f32; 3],
    pub scale: [f32; 3],

    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 4],

    pub light_color: [f32; 4],
    pub light_intensity: f32,
    pub light_range: f32,
    pub light_shadows: bool,
    pub spot_angle: f32,

    pub visible: bool,
}

impl Default for SceneNode {
    fn default() -> Self {
        SceneNode {
            id: 0,
            name: String::from("Node"),
            kind: SceneNodeKind::Empty,
            children: Vec::new(),
            translation: [0.0, 0.0, 0.0],
            rotation_euler: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0, 1.0],
            light_color: [1.0, 1.0, 1.0, 1.0],
            light_intensity: 800.0,
            light_range: 20.0,
            light_shadows: true,
            spot_angle: 45.0,
            visible: true,
        }
    }
}

// ─── Factory ──────────────────────────────────────────────────────────────────

pub fn new_scene_node(id: u64, kind: SceneNodeKind) -> SceneNode {
    let name = match &kind {
        SceneNodeKind::Empty => "Empty".to_string(),
        SceneNodeKind::Mesh(p) => p.to_string(),
        SceneNodeKind::Light(l) => format!("{l} Light"),
    };
    let mut node = SceneNode {
        id,
        name,
        kind: kind.clone(),
        ..Default::default()
    };
    match &kind {
        SceneNodeKind::Mesh(ScenePrimitive::Plane) => {
            node.scale = [5.0, 1.0, 5.0];
            node.color = [0.3, 0.5, 0.3, 1.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Sphere) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Cube) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Cylinder) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Capsule) => {
            node.translation = [0.0, 1.0, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Torus) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Cone) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Mesh(ScenePrimitive::Tetrahedron) => {
            node.translation = [0.0, 0.5, 0.0];
        }
        SceneNodeKind::Light(SceneLightKind::Point) => {
            node.translation = [0.0, 4.0, 0.0];
            node.light_intensity = 800.0;
        }
        SceneNodeKind::Light(SceneLightKind::Directional) => {
            node.rotation_euler = [-45.0, 30.0, 0.0];
            node.light_intensity = 2000.0;
        }
        SceneNodeKind::Light(SceneLightKind::Spot) => {
            node.translation = [0.0, 5.0, 0.0];
            node.rotation_euler = [-90.0, 0.0, 0.0];
            node.light_intensity = 1000.0;
        }
        _ => {}
    }
    node
}

// ─── Resources ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct SceneSelection {
    pub selected: Option<u64>,
}

#[derive(Resource)]
pub struct SceneIdCounter {
    counter: u64,
}

impl Default for SceneIdCounter {
    fn default() -> Self {
        SceneIdCounter { counter: 100 }
    }
}

impl SceneIdCounter {
    pub fn next_id(&mut self) -> u64 {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

#[derive(Resource)]
pub struct SceneChanged {
    pub dirty: bool,
}

impl Default for SceneChanged {
    fn default() -> Self {
        SceneChanged { dirty: true }
    }
}

// ─── Undo / Redo ──────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct UndoHistory {
    pub undo_stack: Vec<Vec<SceneNode>>,
    pub redo_stack: Vec<Vec<SceneNode>>,
    max_history: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        UndoHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

impl UndoHistory {
    pub fn push_snapshot(&mut self, nodes: &[SceneNode]) {
        self.undo_stack.push(nodes.to_vec());
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: &[SceneNode]) -> Option<Vec<SceneNode>> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current.to_vec());
            Some(prev)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current: &[SceneNode]) -> Option<Vec<SceneNode>> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current.to_vec());
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// ─── Editor state ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransformMode {
    Select,
    Translate,
    Rotate,
    Scale,
}

impl Default for TransformMode {
    fn default() -> Self {
        TransformMode::Select
    }
}

impl fmt::Display for TransformMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformMode::Select => write!(f, "Select"),
            TransformMode::Translate => write!(f, "Move"),
            TransformMode::Rotate => write!(f, "Rotate"),
            TransformMode::Scale => write!(f, "Scale"),
        }
    }
}

#[derive(Resource)]
pub struct EditorState {
    pub transform_mode: TransformMode,
    pub scene_file_path: Option<String>,
    pub scene_dirty: bool,
    pub show_grid: bool,
    pub show_stats: bool,
    pub play_mode: bool,
    pub saved_orbit: Option<(f32, f32, f32, [f32; 3])>,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState {
            transform_mode: TransformMode::Select,
            scene_file_path: None,
            scene_dirty: false,
            show_grid: true,
            show_stats: true,
            play_mode: false,
            saved_orbit: None,
        }
    }
}

// ─── Console log ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Resource)]
pub struct ConsoleLog {
    pub entries: Vec<LogEntry>,
    pub show: bool,
    max_entries: usize,
}

impl Default for ConsoleLog {
    fn default() -> Self {
        ConsoleLog {
            entries: vec![LogEntry {
                level: LogLevel::Info,
                message: "Editor initialized.".to_string(),
            }],
            show: true,
            max_entries: 200,
        }
    }
}

impl ConsoleLog {
    pub fn log(&mut self, level: LogLevel, msg: impl Into<String>) {
        self.entries.push(LogEntry {
            level,
            message: msg.into(),
        });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.log(LogLevel::Info, msg);
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.log(LogLevel::Warn, msg);
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        self.log(LogLevel::Error, msg);
    }
}

// ─── Drag-drop state ─────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct DragDropState {
    pub dragging: Option<u64>,
    pub drop_target: Option<DropTarget>,
}

#[derive(Clone, Copy, Debug)]
pub struct DropTarget {
    pub target_id: u64,
    pub position: DropPosition,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropPosition {
    Before,
    Inside,
    After,
}

// ─── Scene document ───────────────────────────────────────────────────────────

#[derive(Resource, Serialize, Deserialize)]
pub struct SceneDocument {
    pub nodes: Vec<SceneNode>,
}

impl Default for SceneDocument {
    fn default() -> Self {
        let ground = SceneNode {
            id: 1,
            name: "Ground".to_string(),
            kind: SceneNodeKind::Mesh(ScenePrimitive::Plane),
            scale: [5.0, 1.0, 5.0],
            color: [0.3, 0.5, 0.3, 1.0],
            ..Default::default()
        };
        let light = SceneNode {
            id: 2,
            name: "Sun".to_string(),
            kind: SceneNodeKind::Light(SceneLightKind::Directional),
            rotation_euler: [-45.0, 30.0, 0.0],
            light_intensity: 2000.0,
            light_shadows: true,
            ..Default::default()
        };
        SceneDocument {
            nodes: vec![ground, light],
        }
    }
}

impl SceneDocument {
    pub fn add_node(&mut self, parent_id: Option<u64>, node: SceneNode) {
        match parent_id {
            None => self.nodes.push(node),
            Some(pid) => {
                if let Some(parent) = self.find_node_mut(pid) {
                    parent.children.push(node);
                } else {
                    self.nodes.push(node);
                }
            }
        }
    }

    pub fn remove_node(&mut self, id: u64) -> bool {
        fn remove_from(nodes: &mut Vec<SceneNode>, id: u64) -> bool {
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
        remove_from(&mut self.nodes, id)
    }

    pub fn take_node(&mut self, id: u64) -> Option<SceneNode> {
        fn take_from(nodes: &mut Vec<SceneNode>, id: u64) -> Option<SceneNode> {
            if let Some(pos) = nodes.iter().position(|n| n.id == id) {
                return Some(nodes.remove(pos));
            }
            for node in nodes.iter_mut() {
                if let Some(found) = take_from(&mut node.children, id) {
                    return Some(found);
                }
            }
            None
        }
        take_from(&mut self.nodes, id)
    }

    pub fn insert_node(&mut self, node: SceneNode, target_id: u64, position: DropPosition) {
        fn insert_in(
            nodes: &mut Vec<SceneNode>,
            node: SceneNode,
            target_id: u64,
            position: DropPosition,
        ) -> bool {
            if let Some(pos) = nodes.iter().position(|n| n.id == target_id) {
                match position {
                    DropPosition::Before => {
                        nodes.insert(pos, node);
                    }
                    DropPosition::After => {
                        nodes.insert(pos + 1, node);
                    }
                    DropPosition::Inside => {
                        nodes[pos].children.push(node);
                    }
                }
                return true;
            }
            for n in nodes.iter_mut() {
                if insert_in(&mut n.children, node.clone(), target_id, position) {
                    return true;
                }
            }
            false
        }
        if !insert_in(&mut self.nodes, node.clone(), target_id, position) {
            self.nodes.push(node);
        }
    }

    pub fn move_node(&mut self, node_id: u64, target_id: u64, position: DropPosition) -> bool {
        if node_id == target_id {
            return false;
        }
        if self.is_ancestor(node_id, target_id) {
            return false;
        }
        if let Some(node) = self.take_node(node_id) {
            self.insert_node(node, target_id, position);
            true
        } else {
            false
        }
    }

    fn is_ancestor(&self, ancestor_id: u64, descendant_id: u64) -> bool {
        fn check(node: &SceneNode, target: u64) -> bool {
            if node.id == target {
                return true;
            }
            node.children.iter().any(|c| check(c, target))
        }
        if let Some(anc) = self.find_node(ancestor_id) {
            check(anc, descendant_id)
        } else {
            false
        }
    }

    pub fn find_node(&self, id: u64) -> Option<&SceneNode> {
        fn find_in(nodes: &[SceneNode], id: u64) -> Option<&SceneNode> {
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
        find_in(&self.nodes, id)
    }

    pub fn find_node_mut(&mut self, id: u64) -> Option<&mut SceneNode> {
        fn find_in_mut(nodes: &mut Vec<SceneNode>, id: u64) -> Option<&mut SceneNode> {
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
        find_in_mut(&mut self.nodes, id)
    }

    pub fn collect_ids(&self) -> Vec<(u64, String)> {
        fn collect(nodes: &[SceneNode], out: &mut Vec<(u64, String)>) {
            for n in nodes {
                out.push((n.id, n.name.clone()));
                collect(&n.children, out);
            }
        }
        let mut out = Vec::new();
        collect(&self.nodes, &mut out);
        out
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
