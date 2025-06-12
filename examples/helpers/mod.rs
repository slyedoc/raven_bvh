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
        .add_systems(Update, cycle_debug);
    }
}

fn cycle_debug(
    //mut commands: Commands,
    mut debug_mode: ResMut<BvhDebugMode>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        #[cfg(not(feature = "tlas"))]
        {
            *debug_mode = match *debug_mode {
                BvhDebugMode::Disabled => BvhDebugMode::Bvhs,
                BvhDebugMode::Bvhs => BvhDebugMode::Disabled,
            };
        };
        #[cfg(feature = "tlas")]
        {
            *debug_mode = match *debug_mode {
            BvhDebugMode::Disabled => BvhDebugMode::Bvhs,
            BvhDebugMode::Bvhs => BvhDebugMode::Tlas,
            BvhDebugMode::Tlas => BvhDebugMode::Disabled,
            };
        }
        
        info!("Debug mode: {:?}", *debug_mode.as_ref());
    }
}
