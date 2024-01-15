mod editor_plugin;

use std::any::TypeId;
use std::f32::consts::PI;

use bevy::ecs::system::SystemState;
use bevy::input::keyboard::KeyboardInput;
use bevy::log;
use bevy::math::DVec2;
use bevy::pbr::{Cascade, CascadeShadowConfigBuilder};
use bevy::reflect::serde::ReflectSerializer;
use bevy::utils::hashbrown::HashMap;
use bevy::utils::{HashSet, Uuid};
use bevy::winit::converters::{convert_element_state, convert_physical_key_code};
use bevy::winit::WindowAndInputEventWriters;
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
};
use editor_plugin::EditorPlugin;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                focused: false,
                decorations: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EditorPlugin)
        .register_type::<Vec<Cascade>>()
        .register_type::<Vec<f32>>()
        .register_type::<HashMap<Entity, Vec<bevy::pbr::Cascade>>>()
        .register_type::<Time>()
        .register_type::<Time<Real>>()
        .register_type::<Time<Virtual>>()
        .register_type::<Time<Fixed>>()
        // .insert_resource(AmbientLight {
        //     brightness: 5.0,
        //     ..default()
        // })
        // .add_plugins(TemporalAntiAliasPlugin)
        // .add_systems(Startup, (setup,))
        // .add_systems(Update, (handle_ipc, send_entities))
        .run();
}

// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     commands.spawn(Camera3dBundle {
//         camera: Camera {
//             hdr: true,
//             ..default()
//         },
//         transform: Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
//         ..default()
//     });
//     // .insert(ScreenSpaceAmbientOcclusionBundle::default())
//     // .insert(TemporalAntiAliasBundle::default());

//     // let material = materials.add(StandardMaterial {
//     //     base_color: Color::rgb(0.5, 0.5, 0.5),
//     //     perceptual_roughness: 1.0,
//     //     reflectance: 0.0,
//     //     ..default()
//     // });

//     // // commands.spawn(asset_server.load(path));

//     commands.spawn(PbrBundle {
//         // mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//         // material: material.clone(),
//         transform: Transform::from_xyz(0.0, 0.0, 1.0),
//         ..default()
//     });
//     // commands.spawn(PbrBundle {
//     //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//     //     material: material.clone(),
//     //     transform: Transform::from_xyz(0.0, -1.0, 0.0),
//     //     ..default()
//     // });
//     // commands.spawn(PbrBundle {
//     //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//     //     material,
//     //     transform: Transform::from_xyz(1.0, 0.0, 0.0),
//     //     ..default()
//     // });
//     // commands.spawn((PbrBundle {
//     //     mesh: meshes.add(Mesh::from(shape::UVSphere {
//     //         radius: 0.4,
//     //         sectors: 72,
//     //         stacks: 36,
//     //     })),
//     //     material: materials.add(StandardMaterial {
//     //         base_color: Color::rgb(0.4, 0.4, 0.4),
//     //         perceptual_roughness: 1.0,
//     //         reflectance: 0.0,
//     //         ..default()
//     //     }),
//     //     ..default()
//     // },));

//     commands.spawn(DirectionalLightBundle {
//         directional_light: DirectionalLight {
//             shadows_enabled: true,
//             ..default()
//         },
//         transform: Transform::from_rotation(Quat::from_euler(
//             EulerRot::ZYX,
//             0.0,
//             PI * -0.15,
//             PI * -0.15,
//         )),
//         ..default()
//     });
// }
