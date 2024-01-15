use std::any::TypeId;

use bevy::{
    ecs::system::{SystemParam, SystemState},
    input::keyboard::KeyboardInput,
    log,
    math::DVec2,
    prelude::*,
    winit::{
        converters::{convert_element_state, convert_physical_key_code},
        WindowAndInputEventWriters,
    },
};

use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};
use roth_shared::{EditorToRuntimeMsg, RuntimeToEditorMsg};

#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    /// Diplays Editor / Editor mode
    #[default]
    Editor,
    /// Play mode, game is being executed
    Game,
}

pub struct EditorPlugin;

struct EditorIpc {
    sender: IpcSender<RuntimeToEditorMsg>,
    receiver: IpcReceiver<EditorToRuntimeMsg>,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        let ipc_server = std::env::args()
            .nth(4)
            .expect("expected ipc server as second argument");

        let (editor_to_runtime_sender, editor_receiver): (
            IpcSender<EditorToRuntimeMsg>,
            IpcReceiver<EditorToRuntimeMsg>,
        ) = ipc_channel::ipc::channel().unwrap();
        let editor_oneshot: IpcSender<(IpcSender<EditorToRuntimeMsg>, String)> =
            IpcSender::connect(ipc_server).unwrap();

        let (server, server_name) =
            IpcOneShotServer::<IpcSender<RuntimeToEditorMsg>>::new().unwrap();
        editor_oneshot
            .send((editor_to_runtime_sender, server_name))
            .unwrap();

        let (_, editor_sender) = server.accept().unwrap();

        app.init_state::<EditorState>()
            .insert_non_send_resource(EditorIpc {
                sender: editor_sender,
                receiver: editor_receiver,
            })
            .add_systems(Update, (handle_ipc,).run_if(in_state(EditorState::Editor)))
            .add_systems(OnEnter(EditorState::Editor), setup_editor)
            .add_systems(OnExit(EditorState::Editor), cleanup_editor);
    }
}

fn handle_ipc(mut world: &mut World) {
    let ipc = world.get_non_send_resource::<EditorIpc>().unwrap();

    let Ok(msg) = ipc.receiver.try_recv() else {
        return;
    };

    let mut system = SystemState::<(
        Query<(Entity, &mut Window)>,
        WindowAndInputEventWriters,
        ResMut<Events<bevy::app::AppExit>>,
        Commands,
        Res<AssetServer>,
    )>::new(&mut world);

    let (mut windows, mut event_writers, mut app_exit_events, mut commands, asset_server) =
        system.get_mut(world);
    let (window_entity, mut window) = windows.single_mut();
    let scale_factor = window.scale_factor();

    match msg {
        EditorToRuntimeMsg::LayoutChange {
            min,
            width,
            height,
            window_position,
        } => {
            window.visible = true;
            window.focused = true;
            window.window_level = bevy::window::WindowLevel::AlwaysOnTop;
            window.resolution.set(width, height);
            let position = IVec2::new(window_position.x, window_position.y);
            let decorations_y_offset = 37;
            let border_offset = 3;
            window.position.set(IVec2::new(
                position.x + (min.0 * scale_factor) as i32 + border_offset,
                position.y + (min.1 * scale_factor) as i32 + decorations_y_offset,
            ));
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
        EditorToRuntimeMsg::Save => {
            let registry = world.resource::<AppTypeRegistry>();

            let entities_without_marker = world.iter_entities().filter_map(|entity| {
                if entity.contains_type_id(TypeId::of::<EditorMarker>()) == false {
                    Some(entity.id())
                } else {
                    None
                }
            });

            let scene = DynamicSceneBuilder::from_world(world)
                .allow_all()
                .deny_resource::<Time>()
                .deny_resource::<Time<Real>>()
                .deny_resource::<Time<Virtual>>()
                .deny_resource::<Time<Fixed>>()
                .deny_resource::<GizmoConfig>()
                .deny::<bevy::window::Window>()
                .deny::<bevy::window::PrimaryWindow>()
                .extract_resources()
                .extract_entities(entities_without_marker)
                .build();

            let ron = match scene.serialize_ron(&registry) {
                Ok(ron) => ron,
                Err(err) => {
                    println!("error serializing scene: {:?}", err);
                    return;
                }
            };

            log::info!("Writing ron bytes: {}", ron.len());
            std::fs::write("./example_bevy/assets/scenes/main.scn.ron", ron).unwrap();
        }
        EditorToRuntimeMsg::LoadScene { path } => {
            commands.spawn(DynamicSceneBundle {
                scene: asset_server.load(path),
                ..default()
            });
        }
        _ => {}
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct EditorMarker;

#[derive(SystemParam)]
struct EditorCommands<'w, 's> {
    commands: Commands<'w, 's>,
}

impl<'w, 's> EditorCommands<'w, 's> {
    fn spawn(&mut self, bundle: impl Bundle) -> &mut Self {
        self.commands.spawn(bundle).insert(EditorMarker);
        self
    }
}

fn setup_editor(
    mut commands: EditorCommands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let cube_mesh_uuid = Uuid::new_v4();
    // meshes.insert(
    //     AssetId::Uuid {
    //         uuid: cube_mesh_uuid,
    //     },
    //     Mesh::from(shape::Cube { size: 1.0 }),
    // );

    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube { size: 1.0 }),
        material: materials.add(Color::rgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn cleanup_editor(mut commands: Commands, query: Query<Entity, With<EditorMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
