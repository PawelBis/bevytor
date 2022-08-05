use crate::editor::EditorStateLabel;
use bevy::app::{App, Plugin};
use bevy::prelude::ExclusiveSystemDescriptorCoercion;

struct EditorScenePlugin;
impl Plugin for EditorScenePlugin {
    fn build(&self, app: &mut App) {
        //app
        //    // This system should start after the assets are initialized
        //    .add_system(select_scene_system
        //        .after(EditorStateLabel::InitializingAssets));
    }
}

fn select_scene_system() {}
