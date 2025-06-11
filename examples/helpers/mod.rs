pub mod camera_free;
pub mod exit;
mod overlay;

use bevy::prelude::*;
use raven_bvh::prelude::*;

pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sly_editor::SlyEditorPlugin::default(), // custom bevy_egui_inspector
            bevy_enhanced_input::EnhancedInputPlugin,
            camera_free::CameraFreePlugin, // camera movement
            exit::ExitPlugin,
        ))
        .add_systems(Update, toggle_debug);
    }
}

fn toggle_debug(
    //mut commands: Commands,
    mut debug: ResMut<BvhDebug>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        debug.enabled = !debug.enabled;
    }
}
