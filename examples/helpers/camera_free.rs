use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub(super) const DEFAULT_SPEED: f32 = 0.1;
pub(super) const DEFAULT_ROTATION: f32 = 0.003;

pub struct CameraFreePlugin;

impl Plugin for CameraFreePlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<CameraFreeInputContext>()
            .add_observer(default_binding)
            .add_observer(apply_movement)
            .add_observer(apply_assend)
            .add_observer(apply_rotate);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[require(
    Name = Name::new("CameraFree"),    
    Actions::<CameraFreeInputContext>,
)]
pub struct CameraFree;

#[derive(Debug, InputContext, Default)]
pub struct CameraFreeInputContext;

fn default_binding(
    trigger: Trigger<Binding<CameraFreeInputContext>>,
    mut cameras: Query<&mut Actions<CameraFreeInputContext>>,
) {
    let mut actions = cameras.get_mut(trigger.target()).unwrap();

    // Movement
    actions.bind::<EnableSprint>().to(KeyCode::ShiftLeft);

    actions
        .bind::<Move>()
        .to((
            Cardinal::wasd_keys().with_conditions_each(BlockBy::<EnableSprint>::default()),
            Cardinal::wasd_keys()
                .with_conditions_each(Chord::<EnableSprint>::default())
                .with_modifiers_each(Scale::splat(10.0)),
        ))
        // Don't trigger the action when the chord is active.
        .with_modifiers((
            //DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(DEFAULT_SPEED), // Additionally multiply by a constant to achieve the desired speed.
        ));

    actions
        .bind::<Assend>()
        .to(Bidirectional {
            positive: KeyCode::KeyQ,
            negative: KeyCode::KeyE,
        })
        .with_modifiers((
            //DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(DEFAULT_SPEED), // Additionally multiply by a constant to achieve the desired speed.
        ));

    actions.bind::<EnableLook>().to(MouseButton::Right);

    actions
        .bind::<Look>()
        .to((
            Input::mouse_motion().with_conditions(Chord::<EnableLook>::default()),
            Axial::right_stick(),
        ))
        //.with_conditions((Chord::<EnableLook>::default(),))
        .with_modifiers((
            //DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            SmoothNudge::default(), // Make movement smooth and independent of the framerate. To only make it framerate-independent, use `DeltaScale`.
            Scale::splat(DEFAULT_ROTATION), // Additionally multiply by a constant to achieve the desired speed.
        ));
}

fn apply_movement(trigger: Trigger<Fired<Move>>, mut query: Query<&mut Transform, With<Camera>>) {
    let mut trans = query.get_mut(trigger.target()).unwrap();
    let change = vec3(trigger.value.x, 0.0, -trigger.value.y);
    let rot = trans.rotation;
    trans.translation += rot * change;
}

fn apply_assend(trigger: Trigger<Fired<Assend>>, mut query: Query<&mut Transform, With<Camera>>) {
    let mut trans = query.get_mut(trigger.target()).unwrap();
    let change = vec3(0.0, trigger.value, 0.0);
    let rot = trans.rotation;
    trans.translation += rot * change;
}

fn apply_rotate(trigger: Trigger<Fired<Look>>, mut query: Query<&mut Transform, With<Camera>>) {
    let change = trigger.value;

    let mut trans = query.get_mut(trigger.target()).unwrap();
    let (mut yaw, mut pitch, _roll) = trans.rotation.to_euler(EulerRot::YXZ);
    yaw -= change.x;
    pitch -= change.y;

    pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

    trans.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
struct Assend;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Look;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EnableLook;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct EnableSprint;
