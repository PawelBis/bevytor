use bevy::prelude::*;
use bevytor::editor::EditorPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorPlugins)
        .run();
}
