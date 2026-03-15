use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Frame, Margin, RichText, ScrollArea};
use bevy_egui::EguiContexts;

use crate::model::{ConsoleLog, LogLevel};

use super::launcher::{AppMode, AppModeRes};

pub fn console_panel_system(
    mut ctx: EguiContexts,
    mut console: ResMut<ConsoleLog>,
    app_mode: Res<AppModeRes>,
    editor: Res<crate::model::EditorState>,
) {
    if app_mode.mode != AppMode::Editor { return; }
    if editor.play_mode { return; }
    let Ok(egui_ctx) = ctx.ctx_mut() else { return };

    if !console.show {
        return;
    }

    egui::TopBottomPanel::bottom("console_panel")
        .default_height(100.0)
        .height_range(40.0..=300.0)
        .resizable(true)
        .show_separator_line(true)
        .frame(Frame {
            fill: Color32::from_rgb(25, 25, 25),
            inner_margin: Margin::symmetric(0, 2),
            ..Default::default()
        })
        .show(egui_ctx, |ui| {
            // Claim full available height so the panel stores the correct
            // resized rect and doesn't snap back to content size.
            ui.set_min_height(ui.available_height());

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new("Console").strong().color(Color32::from_rgb(200, 200, 200)));
                ui.add_space(16.0);
                if ui.small_button("Clear").clicked() {
                    console.entries.clear();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let count = console.entries.len();
                    ui.label(RichText::new(format!("{count} entries")).color(Color32::from_rgb(120, 120, 120)).size(11.0));
                });
            });
            ui.separator();

            ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &console.entries {
                        let (prefix, color) = match entry.level {
                            LogLevel::Info => ("[INFO]", Color32::from_rgb(160, 200, 160)),
                            LogLevel::Warn => ("[WARN]", Color32::from_rgb(230, 200, 80)),
                            LogLevel::Error => ("[ERR] ", Color32::from_rgb(230, 90, 90)),
                        };
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);
                            ui.label(RichText::new(prefix).color(color).size(11.0).monospace());
                            ui.label(RichText::new(&entry.message).color(Color32::from_rgb(190, 190, 190)).size(11.0));
                        });
                    }
                });
        });
}
