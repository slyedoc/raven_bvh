pub mod camera_free;
pub mod exit;
mod overlay;

use bevy::prelude::*;

pub struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sly_editor::SlyEditorPlugin::default(), // custom bevy_egui_inspector
            bevy_enhanced_input::EnhancedInputPlugin,
            camera_free::CameraFreePlugin, // camera movement
            exit::ExitPlugin,
        ));
    }
}
