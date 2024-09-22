use bevy::{
    color::palettes::basic::{GREEN, SILVER},
    input::mouse::MouseMotion,
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        CascadeShadowConfigBuilder,
    },
    prelude::*,
    window::CursorGrabMode,
};
use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;

const WORLD_SEED: u32 = 0xDEADBEEF;
const CHUNK_SIZE: usize = 32;
const BLOCK_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxel Engine".into(),
                    ..default()
                }),
                ..default()
            }),
            PerfUiPlugin,
            WireframePlugin,
        ))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_camera, toggle_wireframe))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    commands.spawn(PerfUiBundle::default());

    let camera_pos = Transform::from_xyz(-25.0, 20.0, -25.0);

    commands.spawn(Camera3dBundle {
        transform: camera_pos.looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10_000.0,
            ..default()
        }
        .into(),
        ..default()
    });

    let stone_material = materials.add(StandardMaterial {
        base_color: Color::from(SILVER),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });
    let dirt_material = materials.add(StandardMaterial {
        base_color: Color::from(Srgba::rgb(0.455, 0.278, 0.0)),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::from(GREEN),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });

    let perlin_noise = Perlin::new(WORLD_SEED);
    let pos = IVec3::ZERO;
    let chunk = generate_chunk(&perlin_noise, &pos);
    commands
        .spawn((
            chunk,
            SpatialBundle {
                transform: Transform {
                    translation: (pos * CHUNK_SIZE as i32).as_vec3(),
                    scale: Vec3::ONE * BLOCK_SIZE,
                    ..Default::default()
                },
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .with_children(|child_builder| {
            // Naive culling, very inefficient
            // TODO: Greedy binary meshing: https://www.youtube.com/watch?v=qnGoGq7DWMc&t=176s
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        if !chunk.is_block_visible(x, y, z) {
                            continue;
                        }

                        let maybe_material = match chunk.blocks[x][y][z] {
                            Block::Stone => Some(stone_material.clone()),
                            Block::Dirt => Some(dirt_material.clone()),
                            Block::Grass => Some(grass_material.clone()),
                            _ => None,
                        };
                        let Some(material) = maybe_material else {
                            continue;
                        };
                        let mesh = meshes.add(Cuboid::from_length(BLOCK_SIZE));
                        let transform = Transform::from_xyz(
                            (x as i32 + pos.x * CHUNK_SIZE as i32) as f32
                                - (0.5 * CHUNK_SIZE as f32),
                            (y as i32 + pos.y * CHUNK_SIZE as i32) as f32
                                - (0.5 * CHUNK_SIZE as f32),
                            (z as i32 + pos.z * CHUNK_SIZE as i32) as f32
                                - (0.5 * CHUNK_SIZE as f32),
                        )
                        .with_scale(Vec3::ONE * BLOCK_SIZE);
                        child_builder.spawn((
                            PbrBundle {
                                mesh,
                                material,
                                transform,
                                ..default()
                            },
                            TerrainMesh,
                        ));
                    }
                }
            }
        });
}

#[derive(Component)]
struct TerrainMesh;

fn generate_chunk(noise: &Perlin, pos: &IVec3) -> Chunk {
    const SAMPLE_SPACING: f64 = 0.05;
    const SCALE: f64 = 15.0;
    let mut blocks = default::<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = ((noise.get([
                (x as i32 + pos.x * CHUNK_SIZE as i32) as f64 * SAMPLE_SPACING,
                (z as i32 + pos.z * CHUNK_SIZE as i32) as f64 * SAMPLE_SPACING,
            ]) * 0.5
                + 0.5)
                * SCALE) as usize
                + 5;
            for y in 0..height - 1 {
                blocks[x][y][z] = Block::Stone;
            }
            blocks[x][height - 1][z] = Block::Dirt;
            blocks[x][height][z] = Block::Grass;
        }
    }
    Chunk { blocks }
}

// 32x32x32 chunk of blocks
#[derive(Component, Clone, Copy)]
struct Chunk {
    // x, y, z
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    fn is_block_visible(&self, x: usize, y: usize, z: usize) -> bool {
        x == 0
            || y == 0
            || z == 0
            || x == CHUNK_SIZE - 1
            || y == CHUNK_SIZE - 1
            || z == CHUNK_SIZE - 1
            || (x > 0 && self.blocks[x - 1][y][z] == Block::Air)
            || (x < CHUNK_SIZE - 1 && self.blocks[x + 1][y][z] == Block::Air)
            || (y > 0 && self.blocks[x][y - 1][z] == Block::Air)
            || (y < CHUNK_SIZE - 1 && self.blocks[x][y + 1][z] == Block::Air)
            || (z > 0 && self.blocks[x][y][z - 1] == Block::Air)
            || (z < CHUNK_SIZE - 1 && self.blocks[x][y][z + 1] == Block::Air)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
}

fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Backquote) {
        wireframe_config.global = !wireframe_config.global;
    }
}

fn move_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
) {
    const CAMERA_VERTICAL_BLOCKS_PER_SECOND: f32 = 7.5;
    const CAMERA_HORIZONTAL_BLOCKS_PER_SECOND: f32 = 15.0;
    for mut transform in q_camera.iter_mut() {
        if keys.pressed(KeyCode::Space) {
            transform.translation.y +=
                CAMERA_VERTICAL_BLOCKS_PER_SECOND * BLOCK_SIZE * time.delta_seconds();
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            transform.translation.y -=
                CAMERA_VERTICAL_BLOCKS_PER_SECOND * BLOCK_SIZE * time.delta_seconds();
        }
        let mut horizontal_movement = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            horizontal_movement.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            horizontal_movement.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            horizontal_movement.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            horizontal_movement.x += 1.0;
        }
        if horizontal_movement != Vec3::ZERO {
            let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let real_horizontal = (Quat::from_rotation_y(yaw) * horizontal_movement).normalize()
                * CAMERA_HORIZONTAL_BLOCKS_PER_SECOND
                * BLOCK_SIZE
                * time.delta_seconds();
            transform.translation += real_horizontal;
        }

        const CAMERA_MOUSE_SENSITIVITY_X: f32 = 0.004;
        const CAMERA_MOUSE_SENSITIVITY_Y: f32 = 0.0025;
        for MouseMotion { delta } in mouse_events.read() {
            transform.rotate_axis(Dir3::NEG_Y, delta.x * CAMERA_MOUSE_SENSITIVITY_X);
            let (yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            pitch = (pitch - delta.y * CAMERA_MOUSE_SENSITIVITY_Y).clamp(-PI * 0.5, PI * 0.5);
            transform.rotation = Quat::from_euler(
                // YXZ order corresponds to the common
                // "yaw"/"pitch"/"roll" convention
                EulerRot::YXZ,
                yaw,
                pitch,
                0.0,
            );
        }
    }
}
