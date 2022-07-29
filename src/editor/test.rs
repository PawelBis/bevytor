use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

pub fn editor_system(mut egui_context: ResMut<EguiContext>) {
    egui::Window::new("Assets").show(egui_context.ctx_mut(), |ui| {
        ui.label("world");
    });
}
