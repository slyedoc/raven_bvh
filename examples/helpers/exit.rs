use bevy::{app::AppExit, prelude::*};

pub struct ExitPlugin;

impl Plugin for ExitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_escape);
    }
}

fn update_escape(input: Res<ButtonInput<KeyCode>>, mut app_exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        app_exit.write(AppExit::Success);
    }
}
