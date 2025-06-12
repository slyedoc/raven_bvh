// use bevy::{
//     diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
//     prelude::*,
// };
// use raven_bvh::BvhStats;

// pub struct OverlayPlugin;

// impl Plugin for OverlayPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins(FrameTimeDiagnosticsPlugin::default())
//             .add_systems(Startup, setup_overlay)
//             .add_systems(Update, update_fps);
//             //.add_system(update_bvh_tri_count)
//             //.add_system(update_render_time)
//             //.add_system(update_ray_count);
//     }
// }

// #[derive(Component)]
// struct FpsText;

// #[derive(Component)]
// struct TriCountText;

// #[derive(Component)]
// struct RenderTimeText;

// #[derive(Component)]
// struct RayCountText;

// const UI_SIZE: f32 = 30.0;
// fn setup_overlay(mut commands: Commands, asset_server: ResMut<AssetServer>) {
//     let ui_font = asset_server.load("fonts/FiraSans-Bold.ttf");

//     commands
//         .spawn_bundle(TextBundle {
//             style: Style {
//                 align_self: AlignSelf::FlexEnd,
//                 position_type: PositionType::Absolute,
//                 position: Rect::<Val> {
//                     bottom: Val::Px(10.0),
//                     right: Val::Px(10.0),
//                     ..Default::default()
//                 },

//                 ..Default::default()
//             },
//             text: Text {
//                 sections: vec![
//                     TextSection {
//                         value: "Tri Count: ".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::WHITE,
//                         },
//                     },
//                     TextSection {
//                         value: "".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::GOLD,
//                         },
//                     },
//                 ],
//                 ..Default::default()
//             },
//             ..Default::default()
//         })
//         .insert(Name::new("ui Tri Count"))
//         .insert(TriCountText);

//     commands
//         .spawn_bundle(TextBundle {
//             style: Style {
//                 align_self: AlignSelf::FlexStart,
//                 position_type: PositionType::Absolute,
//                 position: Rect::<Val> {
//                     bottom: Val::Px(50.0),
//                     left: Val::Px(10.0),
//                     ..Default::default()
//                 },

//                 ..Default::default()
//             },
//             text: Text {
//                 sections: vec![
//                     TextSection {
//                         value: "BVH Render ".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::WHITE,
//                         },
//                     },
//                     TextSection {
//                         value: "".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::GOLD,
//                         },
//                     },
//                 ],
//                 ..Default::default()
//             },
//             ..Default::default()
//         })
//         .insert(Name::new("ui Render Time"))
//         .insert(RenderTimeText);

//         commands
//         .spawn_bundle(TextBundle {
//             style: Style {
//                 align_self: AlignSelf::FlexStart,
//                 position_type: PositionType::Absolute,
//                 position: Rect::<Val> {
//                     bottom: Val::Px(100.0),
//                     left: Val::Px(10.0),
//                     ..Default::default()
//                 },

//                 ..Default::default()
//             },
//             text: Text {
//                 sections: vec![
//                     TextSection {
//                         value: "Rays ".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::WHITE,
//                         },
//                     },
//                     TextSection {
//                         value: "".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::GOLD,
//                         },
//                     },
//                 ],
//                 ..Default::default()
//             },
//             ..Default::default()
//         })
//         .insert(Name::new("ui Ray Count"))
//         .insert(RayCountText);

//     commands
//         .spawn_bundle(TextBundle {
//             style: Style {
//                 position_type: PositionType::Absolute,
//                 position: Rect::<Val> {
//                     left: Val::Px(10.0),
//                     bottom: Val::Px(10.0),
//                     ..Default::default()
//                 },
//                 align_self: AlignSelf::FlexEnd,
//                 ..Default::default()
//             },
//             // Use `Text` directly
//             text: Text {
//                 // Construct a `Vec` of `TextSection`s
//                 sections: vec![
//                     TextSection {
//                         value: "FPS: ".to_string(),
//                         style: TextStyle {
//                             font: ui_font.clone(),
//                             font_size: UI_SIZE,
//                             color: Color::WHITE,
//                         },
//                     },
//                     TextSection {
//                         value: "".to_string(),
//                         style: TextStyle {
//                             font: ui_font,
//                             font_size: UI_SIZE,
//                             color: Color::GOLD,
//                         },
//                     },
//                 ],
//                 ..Default::default()
//             },
//             ..Default::default()
//         })
//         .insert(Name::new("ui FPS"))
//         .insert(FpsText);
// }

// #[allow(dead_code)]
// fn update_bvh_tri_count(mut query: Query<&mut Text, With<TriCountText>>, stats: Res<BvhStats>) {
//     for mut text in query.iter_mut() {
//         // Update the value of the second section
//         text.sections[1].value = stats.tri_count.to_string();
//     }
// }

// #[allow(dead_code)]
// fn update_render_time(mut query: Query<&mut Text, With<RenderTimeText>>, stats: Res<BvhStats>) {
//     for mut text in query.iter_mut() {
//         // Update the value of the second section
//         text.sections[1].value = format!("{:.2} ms", stats.camera_time.as_millis());
//     }
// }

// #[allow(dead_code)]
// fn update_ray_count(mut query: Query<&mut Text, With<RayCountText>>, stats: Res<BvhStats>) {
//     for mut text in query.iter_mut() {
//         // Update the value of the second section
//         text.sections[1].value = format!("{:.0} Mps", stats.ray_count as f32 / stats.camera_time.as_micros() as f32);
//     }
// }
