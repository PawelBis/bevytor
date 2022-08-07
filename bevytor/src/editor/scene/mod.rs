// use crate::editor::EditorStateLabel;
use bevy::app::{App, Plugin};
// use bevy::prelude::ExclusiveSystemDescriptorCoercion;
pub mod scene_picker;

struct EditorScenePlugin;
impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_scene_system);
        //app
        //    // This system should start after the assets are initialized
        //    .add_system(select_scene_system
        //        .after(EditorStateLabel::InitializingAssets));
    }
}

fn select_scene_system() {}
