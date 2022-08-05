use crate::editor::assets::asset_loader::AssetDirectory;
use bevy_egui::egui::collapsing_header::CollapsingState;
use bevy_egui::egui::{
    Color32, FontId, Image, Rect, Response, Rounding, Sense, TextureId, Ui, Vec2, Widget,
    WidgetInfo, WidgetType,
};
use std::default::Default;
use std::path::PathBuf;
use bevy::asset::Asset;
use crate::editor::ui::asset_browser::Selection;

#[derive(Default)]
pub struct Thumbnail {
    pub size: Vec2,
    pub texture_id: TextureId,
    pub label: String,
    pub selected: bool,
}

impl Widget for Thumbnail {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let image = Image::new(self.texture_id, self.size);
        let sense = Sense::click();
        let frame = true;
        let selected = self.selected;
        let label_size = ui
            .painter()
            .layout_no_wrap(self.label.clone(), FontId::default(), Color32::default())
            .rect
            .size();
        let label_width = label_size.x;
        let label_height = label_size.y;

        let padding = if frame {
            // so we can see that it is a button:
            Vec2::splat(ui.spacing().button_padding.x)
        } else {
            Vec2::ZERO
        };
        let padded_size = image.size() + 2.0 * padding;
        let (rect, mut response) =
            ui.allocate_exact_size(padded_size + Vec2::new(0.0, label_height), sense);
        let image_rect = Rect::from_min_max(rect.min, rect.max - Vec2::new(0.0, label_height));
        response.widget_info(|| WidgetInfo::new(WidgetType::ImageButton));

        if ui.is_rect_visible(rect) {
            let (expansion, rounding, fill, stroke, font_color) = if selected {
                let selection = ui.visuals().selection;
                (
                    -padding,
                    Rounding::none(),
                    selection.bg_fill,
                    selection.stroke,
                    selection.stroke.color,
                )
            } else if frame {
                let visuals = ui.style().interact(&response);
                let expansion = if response.hovered {
                    Vec2::splat(visuals.expansion) - padding
                } else {
                    Vec2::splat(visuals.expansion)
                };
                (
                    expansion,
                    visuals.rounding,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                    visuals.fg_stroke.color,
                )
            } else {
                Default::default()
            };

            // Draw frame background (for transparent images):
            ui.painter()
                .rect_filled(image_rect.expand2(expansion), rounding, fill);

            let image_inner_rect = ui
                .layout()
                .align_size_within_rect(image.size(), image_rect.shrink2(padding));
            image.paint_at(ui, image_inner_rect);

            // Draw frame outline:
            ui.painter()
                .rect_stroke(image_rect.expand2(expansion), rounding, stroke);

            let label_part = rect.width() / label_width;
            if label_part < 1.0 {
                // Display whole label as tooltip if width of the thumbnail doesn't allow for displaying it
                response = response.on_hover_text(self.label.clone());
                let new_len_raw = (self.label.chars().count() as f32 * label_part).trunc() as usize;
                let new_len = if new_len_raw > 2 {
                    new_len_raw - 2
                } else {
                    new_len_raw
                };
                let idx = self.label.char_indices().nth(new_len).unwrap().0;
                self.label.truncate(idx);
                self.label.push_str("...")
            };
            let label = ui
                .painter()
                .layout_no_wrap(self.label, FontId::default(), font_color);
            let label_pos = rect.left_bottom() - Vec2::new(0.0, label_height);
            ui.painter().galley(label_pos, label);
        }

        response
    }
}

pub fn thumbnail(ui: &mut Ui, label: String, size: Vec2, texture_id: TextureId) -> Response {
    let image_button = Thumbnail {
        label: label.clone(),
        size,
        texture_id,
        selected: false,
        ..Default::default()
    };
    ui.add(image_button)
}

fn draw_directories(
    ui: &mut Ui,
    directory: &AssetDirectory,
    draw_assets: bool,
) -> Option<Selection> {
    for child in directory.children_directories.iter() {
        if child.children_directories.is_empty() {
            if ui.button(child.get_name()).clicked() {
                return Some(Selection::Directory(child.clone()));
            }
        } else {
            if let Some(dir) = draw_directory_hierarchy(ui, child, draw_assets) {
                return Some(dir);
            }
        }
    }
    None
}

fn draw_assets(ui: &mut Ui, directory: &AssetDirectory) -> Option<Selection> {
    for asset in directory.assets.iter() {
        if ui.button(asset.get_name()).clicked() {
            return Some(Selection::Asset(asset.clone()));
        }
    }
    None
}

/// Draws tree like structure of asset_directory
/// selected path if selection took place
pub fn draw_directory_hierarchy(
    ui: &mut Ui,
    asset_directory: &AssetDirectory,
    should_draw_assets: bool,
) -> Option<Selection> {
    let directory_name = asset_directory.get_name();
    let id = ui.make_persistent_id(&directory_name);
    let mut new_selection: Option<Selection> = None;
    CollapsingState::load_with_default_open(ui.ctx(), id, false)
        .show_header(ui, |ui| {
            let response = ui.button(directory_name);
            if response.clicked() {
                new_selection = Some(Selection::Directory(asset_directory.clone()));
            }
        })
        .body(|ui| {
            if let Some(selection) = draw_directories(ui, asset_directory, should_draw_assets) {
                new_selection = Some(selection);
            }

            if !should_draw_assets {
                return;
            }
            if let Some(selection) = draw_assets(ui, asset_directory) {
                new_selection = Some(selection)
            }
        });

    new_selection
}
