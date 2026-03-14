use bevy::prelude::*;

use crate::model::{
    EditorAlignItems, EditorChanged, EditorDocument, EditorFlexDirection, EditorJustifyContent,
    EditorNodeType, EditorOverflow, EditorPositionType, EditorSelection, EditorUiNode, EditorVal,
};

#[derive(Component)]
pub struct PreviewUiRoot;

#[derive(Component)]
pub struct PreviewUiNode(pub u64);

pub fn viewport_sync_system(
    mut commands: Commands,
    mut changed: ResMut<EditorChanged>,
    document: Res<EditorDocument>,
    selection: Res<EditorSelection>,
    preview_roots: Query<Entity, With<PreviewUiRoot>>,
) {
    if !changed.dirty {
        return;
    }
    changed.dirty = false;

    // Despawn all existing preview entities
    for entity in &preview_roots {
        commands.entity(entity).despawn();
    }

    // Re-spawn from document
    for root in &document.roots {
        spawn_root_node(&mut commands, root, selection.selected);
    }
}

fn editor_val_to_bevy(val: &EditorVal) -> Val {
    match val {
        EditorVal::Auto => Val::Auto,
        EditorVal::Px(v) => Val::Px(*v),
        EditorVal::Percent(v) => Val::Percent(*v),
        EditorVal::Vw(v) => Val::Vw(*v),
        EditorVal::Vh(v) => Val::Vh(*v),
    }
}

fn editor_rect_to_bevy(rect: &crate::model::EditorRect) -> UiRect {
    UiRect {
        top: editor_val_to_bevy(&rect.top),
        right: editor_val_to_bevy(&rect.right),
        bottom: editor_val_to_bevy(&rect.bottom),
        left: editor_val_to_bevy(&rect.left),
    }
}

fn editor_flex_dir(d: &EditorFlexDirection) -> FlexDirection {
    match d {
        EditorFlexDirection::Row => FlexDirection::Row,
        EditorFlexDirection::Column => FlexDirection::Column,
        EditorFlexDirection::RowReverse => FlexDirection::RowReverse,
        EditorFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
    }
}

fn editor_justify(j: &EditorJustifyContent) -> JustifyContent {
    match j {
        EditorJustifyContent::FlexStart => JustifyContent::FlexStart,
        EditorJustifyContent::FlexEnd => JustifyContent::FlexEnd,
        EditorJustifyContent::Center => JustifyContent::Center,
        EditorJustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
        EditorJustifyContent::SpaceAround => JustifyContent::SpaceAround,
        EditorJustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

fn editor_align(a: &EditorAlignItems) -> AlignItems {
    match a {
        EditorAlignItems::FlexStart => AlignItems::FlexStart,
        EditorAlignItems::FlexEnd => AlignItems::FlexEnd,
        EditorAlignItems::Center => AlignItems::Center,
        EditorAlignItems::Stretch => AlignItems::Stretch,
        EditorAlignItems::Baseline => AlignItems::Baseline,
    }
}

fn editor_position(p: &EditorPositionType) -> PositionType {
    match p {
        EditorPositionType::Relative => PositionType::Relative,
        EditorPositionType::Absolute => PositionType::Absolute,
    }
}

fn editor_overflow(o: &EditorOverflow) -> OverflowAxis {
    match o {
        EditorOverflow::Visible => OverflowAxis::Visible,
        EditorOverflow::Clip => OverflowAxis::Clip,
        EditorOverflow::Hidden => OverflowAxis::Hidden,
    }
}

fn build_node_bundle(node: &EditorUiNode, is_selected: bool) -> (Node, BackgroundColor, BorderColor, Visibility) {
    let [r, g, b, a] = node.background_color;
    let [br, bg2, bb, ba] = node.border_color;

    let border_color = if is_selected {
        BorderColor::all(Color::srgba(0.2, 0.5, 1.0, 1.0))
    } else {
        BorderColor::all(Color::srgba(br, bg2, bb, ba))
    };

    let node_comp = Node {
        width: editor_val_to_bevy(&node.width),
        height: editor_val_to_bevy(&node.height),
        flex_direction: editor_flex_dir(&node.flex_direction),
        justify_content: editor_justify(&node.justify_content),
        align_items: editor_align(&node.align_items),
        position_type: editor_position(&node.position_type),
        padding: editor_rect_to_bevy(&node.padding),
        margin: editor_rect_to_bevy(&node.margin),
        border: editor_rect_to_bevy(&node.border),
        flex_wrap: if node.flex_wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        },
        flex_grow: node.flex_grow,
        flex_shrink: node.flex_shrink,
        flex_basis: editor_val_to_bevy(&node.flex_basis),
        row_gap: editor_val_to_bevy(&node.row_gap),
        column_gap: editor_val_to_bevy(&node.column_gap),
        overflow: Overflow {
            x: editor_overflow(&node.overflow_x),
            y: editor_overflow(&node.overflow_y),
        },
        border_radius: BorderRadius::all(Val::Px(node.border_radius)),
        ..default()
    };

    let bg = BackgroundColor(Color::srgba(r, g, b, a));
    let vis = if node.visible { Visibility::Visible } else { Visibility::Hidden };

    (node_comp, bg, border_color, vis)
}

fn spawn_root_node(commands: &mut Commands, node: &EditorUiNode, selected: Option<u64>) {
    let is_selected = selected == Some(node.id);
    let (node_comp, bg, bc, vis) = build_node_bundle(node, is_selected);

    let mut entity_cmds = commands.spawn((
        node_comp,
        bg,
        bc,
        vis,
        PreviewUiRoot,
        PreviewUiNode(node.id),
    ));

    if is_selected {
        entity_cmds.insert(Outline {
            width: Val::Px(2.0),
            color: Color::srgba(0.2, 0.5, 1.0, 0.8),
            ..default()
        });
    }

    match node.node_type {
        EditorNodeType::Text => {
            let [r, g, b, a] = node.text_color;
            entity_cmds.insert((
                Text::new(node.text_content.clone()),
                TextFont {
                    font_size: node.font_size,
                    ..default()
                },
                TextColor(Color::srgba(r, g, b, a)),
            ));
        }
        EditorNodeType::Button => {
            entity_cmds.insert(Button);
            let [r, g, b, a] = node.text_color;
            let label = node.text_content.clone();
            let font_size = node.font_size;
            entity_cmds.with_children(|cb| {
                cb.spawn((
                    Text::new(label),
                    TextFont {
                        font_size,
                        ..default()
                    },
                    TextColor(Color::srgba(r, g, b, a)),
                ));
            });
        }
        _ => {}
    }

    let entity_id = entity_cmds.id();
    for child in &node.children {
        spawn_child_node(commands, child, entity_id, selected);
    }
}

fn spawn_child_node(
    commands: &mut Commands,
    node: &EditorUiNode,
    parent: Entity,
    selected: Option<u64>,
) {
    let is_selected = selected == Some(node.id);
    let (node_comp, bg, bc, vis) = build_node_bundle(node, is_selected);

    let mut child_id = Entity::PLACEHOLDER;
    commands.entity(parent).with_children(|cb| {
        let mut ec = cb.spawn((
            node_comp,
            bg,
            bc,
            vis,
            PreviewUiNode(node.id),
        ));

        if is_selected {
            ec.insert(Outline {
                width: Val::Px(2.0),
                color: Color::srgba(0.2, 0.5, 1.0, 0.8),
                ..default()
            });
        }

        match node.node_type {
            EditorNodeType::Text => {
                let [r, g, b, a] = node.text_color;
                ec.insert((
                    Text::new(node.text_content.clone()),
                    TextFont {
                        font_size: node.font_size,
                        ..default()
                    },
                    TextColor(Color::srgba(r, g, b, a)),
                ));
            }
            EditorNodeType::Button => {
                ec.insert(Button);
                let [r, g, b, a] = node.text_color;
                let label = node.text_content.clone();
                let font_size = node.font_size;
                ec.with_children(|btn| {
                    btn.spawn((
                        Text::new(label),
                        TextFont {
                            font_size,
                            ..default()
                        },
                        TextColor(Color::srgba(r, g, b, a)),
                    ));
                });
            }
            _ => {}
        }

        child_id = ec.id();
    });

    for grandchild in &node.children {
        spawn_child_node(commands, grandchild, child_id, selected);
    }
}

