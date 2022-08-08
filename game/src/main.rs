use bevy::prelude::*;
use bevytor::editor::{EditorPlugins, EditorStateLabel};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(EditorStateLabel::InitializingAssets)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(EditorPlugins)
        .run();
}
