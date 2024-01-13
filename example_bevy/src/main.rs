//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::any::TypeId;
use std::f32::consts::PI;

use bevy::input::keyboard::KeyboardInput;
use bevy::math::DVec2;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::utils::Uuid;
use bevy::winit::converters::{convert_element_state, convert_physical_key_code};
use bevy::winit::WindowAndInputEventWriters;
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
};
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};
struct EditorIpc {
    sender: IpcSender<RuntimeToEditorMsg>,
    receiver: IpcReceiver<EditorToRuntimeMsg>,
}

fn main() {
    let ipc_server = std::env::args()
        .nth(4)
        .expect("expected ipc server as second argument");

    let (editor_to_runtime_sender, editor_receiver): (
        IpcSender<EditorToRuntimeMsg>,
        IpcReceiver<EditorToRuntimeMsg>,
    ) = ipc_channel::ipc::channel().unwrap();
    let editor_oneshot: IpcSender<(IpcSender<EditorToRuntimeMsg>, String)> =
        IpcSender::connect(ipc_server).unwrap();

    let (server, server_name) = IpcOneShotServer::<IpcSender<RuntimeToEditorMsg>>::new().unwrap();
    editor_oneshot
        .send((editor_to_runtime_sender, server_name))
        .unwrap();

    let (_, editor_sender) = server.accept().unwrap();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                focused: false,
                decorations: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(AmbientLight {
            brightness: 5.0,
            ..default()
        })
        // .add_plugins(TemporalAntiAliasPlugin)
        .insert_non_send_resource(EditorIpc {
            sender: editor_sender,
            receiver: editor_receiver,
        })
        // .register_type::<Camera3dBundle>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_ipc))
        .run();
}

fn handle_ipc(
    mut windows: Query<(Entity, &mut Window)>,
    ipc: NonSend<EditorIpc>,
    mut event_writers: WindowAndInputEventWriters,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    let (window_entity, mut window) = windows.single_mut();
    let scale_factor = window.scale_factor();

    let Ok(msg) = ipc.receiver.try_recv() else {
        return;
    };

    match msg {
        EditorToRuntimeMsg::LayoutChange {
            min,
            width,
            height,
            window_position,
        } => {
            window.visible = true;
            window.focused = true;
            window.resolution.set(width, height);
            let position = IVec2::new(window_position.x, window_position.y);
            let decorations_y_offset = 37;
            let border_offset = 3;
            window.position.set(IVec2::new(
                position.x + (min.0 * scale_factor) as i32 + border_offset,
                position.y + (min.1 * scale_factor) as i32 + decorations_y_offset,
            ));
            // window.focused = true;
        }
        EditorToRuntimeMsg::CursorMoved { position } => {
            let physical_position = DVec2::new(position.x, position.y);
            window.set_physical_cursor_position(Some(physical_position));
            event_writers.cursor_moved.send(CursorMoved {
                window: window_entity,
                position: (physical_position / window.resolution.scale_factor() as f64).as_vec2(),
            });
        }
        EditorToRuntimeMsg::CursorEntered { .. } => {
            event_writers.cursor_entered.send(CursorEntered {
                window: window_entity,
            });
        }
        EditorToRuntimeMsg::CursorLeft { .. } => {
            event_writers.cursor_left.send(CursorLeft {
                window: window_entity,
            });
        }
        EditorToRuntimeMsg::KeyboardInput {
            state,
            physical_key,
        } => {
            event_writers.keyboard_input.send(KeyboardInput {
                state: convert_element_state(state),
                key_code: convert_physical_key_code(physical_key),
                window: window_entity,
            });
        }
        EditorToRuntimeMsg::ReceivedCharacter { char } => {
            event_writers.character_input.send(ReceivedCharacter {
                char,
                window: window_entity,
            });
        }
        EditorToRuntimeMsg::Shutdown => {
            app_exit_events.send(bevy::app::AppExit);
        }
        _ => {}
    }
}

#[derive(Component, Reflect)]
struct TestComponent {
    mesh: Mesh,
}

fn send_entities(world: &mut World) {
    {
        // let registry = world.resource::<AppTypeRegistry>();
        // let registry = registry.read();
        // let p = registry.get(TypeId::of::<Camera3dBundle>());
        // println!("p: {:?}", p);
        // for t in registry.iter() {
        //     println!("type: {:?}", t);
        // }
        // let bundles = world.bundles();
        // let bundle_id = bundles.get_id(TypeId::of::<Camera3dBundle>());
        // println!("bundle_id: {:?}", bundle_id);
    }

    {
        // let mut meshes = world.resource_mut::<Assets<Mesh>>();
        // let asset_id = AssetId::<Mesh>::Uuid {
        //     uuid: Uuid::new_v4(),
        // };
        // meshes.insert(asset_id, Mesh::from(shape::Cube { size: 1.0 }));
        // // let handle = meshes.get(asset_id).unwrap();
        // drop(meshes);
        // let registry = world.resource::<AppTypeRegistry>();
        // let mut scene_world = World::new();
        // scene_world.insert_resource(registry.clone());
        // scene_world.spawn((Camera3dBundle::default(),));

        // scene_world.spawn((PbrBundle {
        //     mesh: Handle::Weak(asset_id),
        //     ..default()
        // },));
        // scene_world.spawn(DirectionalLightBundle {
        //     directional_light: DirectionalLight {
        //         shadows_enabled: true,
        //         ..default()
        //     },
        //     transform: Transform {
        //         translation: Vec3::new(0.0, 2.0, 0.0),
        //         rotation: Quat::from_rotation_x(-PI / 4.),
        //         ..default()
        //     },
        //     // The default cascade config is designed to handle large scenes.
        //     // As this example has a much smaller world, we can tighten the shadow
        //     // bounds for better visual quality.
        //     cascade_shadow_config: CascadeShadowConfigBuilder {
        //         first_cascade_far_bound: 4.0,
        //         maximum_distance: 10.0,
        //         ..default()
        //     }
        //     .into(),
        //     ..default()
        // });
        // let scene = DynamicScene::from_world(&world);
        // let ron = scene.serialize_ron(&registry).unwrap();
        // std::fs::write("scene.ron", ron).unwrap();
        // let server = world.resource::<AssetServer>();

        // server.
    }

    // let mut entities = world.query::<Entity>();
    // let mut entities = entities.iter(&world).collect::<Vec<_>>();
    // entities.sort_by_key(|entity| *entity);
    // // let entities = entities
    // //     .iter()
    // //     .map(|(entity, debug_name)| (*entity, debug_name.name.clone()))
    // //     .collect::<Vec<_>>();

    // let ipc = world.get_non_send_resource::<EditorIpc>().unwrap();

    // ipc.sender
    //     .send(RuntimeToEditorMsg::Entities { entities })
    //     .unwrap();

    // let mut sender = ipc.sender.clone();
    // let msg = RuntimeToEditorMsg::Entities(entities);
    // sender.send(msg).unwrap();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera3dBundle {
        camera: Camera {
            hdr: true,
            ..default()
        },
        transform: Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // .insert(ScreenSpaceAmbientOcclusionBundle::default())
    // .insert(TemporalAntiAliasBundle::default());

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.5),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    });

    // // commands.spawn(asset_server.load(path));

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..default()
    });
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: material.clone(),
    //     transform: Transform::from_xyz(0.0, -1.0, 0.0),
    //     ..default()
    // });
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material,
    //     transform: Transform::from_xyz(1.0, 0.0, 0.0),
    //     ..default()
    // });
    // commands.spawn((PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::UVSphere {
    //         radius: 0.4,
    //         sectors: 72,
    //         stacks: 36,
    //     })),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::rgb(0.4, 0.4, 0.4),
    //         perceptual_roughness: 1.0,
    //         reflectance: 0.0,
    //         ..default()
    //     }),
    //     ..default()
    // },));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI * -0.15,
            PI * -0.15,
        )),
        ..default()
    });
}
