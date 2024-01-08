//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::f32::consts::PI;

use bevy::input::keyboard::KeyboardInput;
use bevy::math::DVec2;
use bevy::window::Cursor;
use bevy::winit::converters::{convert_element_state, convert_physical_key_code};
use bevy::winit::WindowAndInputEventWriters;
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
    render::camera::TemporalJitter,
};
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};
struct EditorIpc {
    sender: IpcSender<RuntimeToEditorMsg>,
    receiver: IpcReceiver<EditorToRuntimeMsg>,
}

fn main() {
    // let window_id = std::env::args()
    //     .nth(2)
    //     .expect("expected window id as first argument");

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
        .add_plugins(TemporalAntiAliasPlugin)
        .insert_non_send_resource(EditorIpc {
            sender: editor_sender,
            receiver: editor_receiver,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_ipc, send_entities))
        .add_systems(Update, update)
        .run();
}

fn handle_ipc(
    mut windows: Query<(Entity, &mut Window)>,
    ipc: NonSend<EditorIpc>,
    mut event_writers: WindowAndInputEventWriters,
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
        _ => {}
    }
}

fn send_entities(world: &mut World) {
    let mut entities = world.query::<Entity>();
    let mut entities = entities.iter(&world).collect::<Vec<_>>();
    entities.sort_by_key(|entity| *entity);
    // let entities = entities
    //     .iter()
    //     .map(|(entity, debug_name)| (*entity, debug_name.name.clone()))
    //     .collect::<Vec<_>>();

    let ipc = world.get_non_send_resource::<EditorIpc>().unwrap();

    ipc.sender
        .send(RuntimeToEditorMsg::Entities { entities })
        .unwrap();

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
    commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.5),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material,
        transform: Transform::from_xyz(1.0, 0.0, 0.0),
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.4,
                sectors: 72,
                stacks: 36,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.4, 0.4, 0.4),
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                ..default()
            }),
            ..default()
        },
        SphereMarker,
    ));

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

    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 26.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn update(
    camera: Query<
        (
            Entity,
            Option<&ScreenSpaceAmbientOcclusionSettings>,
            Option<&TemporalJitter>,
        ),
        With<Camera>,
    >,
    mut text: Query<&mut Text>,
    mut sphere: Query<&mut Transform, With<SphereMarker>>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut sphere = sphere.single_mut();
    sphere.translation.y = (time.elapsed_seconds() / 1.7).sin() * 0.7;

    let (camera_entity, ssao_settings, temporal_jitter) = camera.single();

    let mut commands = commands.entity(camera_entity);
    if keycode.just_pressed(KeyCode::Digit1) {
        commands.remove::<ScreenSpaceAmbientOcclusionSettings>();
    }
    if keycode.just_pressed(KeyCode::Digit2) {
        commands.insert(ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Low,
        });
    }
    if keycode.just_pressed(KeyCode::Digit3) {
        commands.insert(ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
        });
    }
    if keycode.just_pressed(KeyCode::Digit4) {
        commands.insert(ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
        });
    }
    if keycode.just_pressed(KeyCode::Digit5) {
        commands.insert(ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
        });
    }
    if keycode.just_pressed(KeyCode::Space) {
        if temporal_jitter.is_some() {
            commands.remove::<TemporalJitter>();
        } else {
            commands.insert(TemporalJitter::default());
        }
    }

    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;
    text.clear();

    let (o, l, m, h, u) = match ssao_settings.map(|s| s.quality_level) {
        None => ("*", "", "", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Low) => ("", "*", "", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Medium) => ("", "", "*", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::High) => ("", "", "", "*", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Ultra) => ("", "", "", "", "*"),
        _ => unreachable!(),
    };

    text.push_str("SSAO Quality:\n");
    text.push_str(&format!("(1) {o}Off{o}\n"));
    text.push_str(&format!("(2) {l}Low{l}\n"));
    text.push_str(&format!("(3) {m}Medium{m}\n"));
    text.push_str(&format!("(4) {h}High{h}\n"));
    text.push_str(&format!("(5) {u}Ultra{u}\n\n"));

    text.push_str("Temporal Antialiasing:\n");
    text.push_str(match temporal_jitter {
        Some(_) => "(Space) Enabled",
        None => "(Space) Disabled",
    });
}

#[derive(Component)]
struct SphereMarker;
