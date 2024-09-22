use bevy::{
    color::palettes::basic::{GREEN, SILVER},
    input::mouse::MouseMotion,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    window::CursorGrabMode,
};
use iyes_perf_ui::{entries::PerfUiBundle, prelude::*};
use noise::{NoiseFn, Perlin};
use std::{collections::HashSet, f32::consts::PI};

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
        .insert_resource(WorldGenNoise(Perlin::new(WORLD_SEED)))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_systems(Startup, (initialize_block_assets, setup).chain())
        .add_systems(
            Update,
            (
                move_camera,
                toggle_wireframe,
                update_loaded_chunks,
                update_camera_chunk_position,
            ),
        )
        .run();
}

#[derive(Resource)]
struct WorldGenNoise(Perlin);

#[derive(Resource)]
struct BlockMaterials {
    stone: Handle<StandardMaterial>,
    dirt: Handle<StandardMaterial>,
    grass: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct BlockMeshes {
    cube: Handle<Mesh>,
}

fn initialize_block_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block_meshes = BlockMeshes {
        cube: meshes.add(Cuboid::from_length(BLOCK_SIZE)),
    };
    commands.insert_resource(block_meshes);

    let stone = materials.add(StandardMaterial {
        base_color: Color::from(SILVER),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });
    let dirt = materials.add(StandardMaterial {
        base_color: Color::from(Srgba::rgb(0.455, 0.278, 0.0)),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });
    let grass = materials.add(StandardMaterial {
        base_color: Color::from(GREEN),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..Default::default()
    });
    commands.insert_resource(BlockMaterials { stone, dirt, grass });
}

fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    commands.spawn(PerfUiBundle::default());

    let camera_pos = Transform::from_xyz(0.0, 60.0, 0.0);

    commands.spawn((
        Camera3dBundle {
            transform: camera_pos.looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        ChunkPosition::default(),
    ));

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
        ..default()
    });
}

fn update_loaded_chunks(
    mut commands: Commands,
    q_camera_position: Query<&GlobalTransform, (With<Camera3d>, Changed<ChunkPosition>)>,
    q_chunk_position: Query<(Entity, &ChunkPosition), With<Chunk>>,
    block_meshes: Res<BlockMeshes>,
    block_materials: Res<BlockMaterials>,
    world_gen_noise: Res<WorldGenNoise>,
) {
    let Ok(pos) = q_camera_position.get_single() else {
        return;
    };
    let camera_position = pos.compute_transform().translation;
    let chunk_pos = ChunkPosition::from_world_position(&camera_position);
    // Determine position of chunks that should be loaded
    let mut should_be_loaded_positions: HashSet<IVec3> = HashSet::new();
    const LOAD_DISTANCE_CHUNKS: i32 = 5;
    for chunk_x in -LOAD_DISTANCE_CHUNKS..=LOAD_DISTANCE_CHUNKS {
        for chunk_y in 0..1 {
            for chunk_z in -LOAD_DISTANCE_CHUNKS..=LOAD_DISTANCE_CHUNKS {
                let cur_chunk_pos =
                    ChunkPosition(chunk_pos.0.with_y(0) + IVec3::new(chunk_x, chunk_y, chunk_z));
                should_be_loaded_positions.insert(cur_chunk_pos.0);
            }
        }
    }
    // Iterate over loaded chunks, despawning any which shouldn't be loaded right now
    // By removing loaded chunks from our hash set, we are left only with the chunks
    // which need to be loaded.
    for (entity, chunk_pos) in q_chunk_position.iter() {
        if !should_be_loaded_positions.remove(&chunk_pos.0) {
            // The chunk shouldn't be loaded since it's not in our set
            commands.entity(entity).despawn_recursive();
        }
    }
    // Finally, load the new chunks
    for pos in should_be_loaded_positions {
        let chunk = generate_chunk(&world_gen_noise.0, &pos);
        commands
            .spawn((
                chunk,
                ChunkPosition(pos),
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
                                Block::Stone => Some(block_materials.stone.clone()),
                                Block::Dirt => Some(block_materials.dirt.clone()),
                                Block::Grass => Some(block_materials.grass.clone()),
                                _ => None,
                            };
                            let Some(material) = maybe_material else {
                                continue;
                            };
                            let mesh = block_meshes.cube.clone();
                            let transform = Transform::from_xyz(x as f32, y as f32, z as f32);
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
}

fn update_camera_chunk_position(
    mut q_camera: Query<(&mut ChunkPosition, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((mut chunk_pos, transform)) = q_camera.get_single_mut() else {
        return;
    };
    let new_chunk_pos = ChunkPosition::from_world_position(&transform.translation());
    if new_chunk_pos != *chunk_pos {
        chunk_pos.0 = new_chunk_pos.0;
    }
}

#[derive(Component)]
struct TerrainMesh;

fn generate_chunk(noise: &Perlin, chunk_pos: &IVec3) -> Chunk {
    const SAMPLE_SPACING: f64 = 0.025;
    const SCALE: f64 = 60.0;
    let mut blocks = default::<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = ((noise.get([
                (x as i32 + chunk_pos.x * CHUNK_SIZE as i32) as f64 * SAMPLE_SPACING,
                (z as i32 + chunk_pos.z * CHUNK_SIZE as i32) as f64 * SAMPLE_SPACING,
            ]) * 0.5
                + 0.5)
                * SCALE) as i32
                + 1;
            let Some(chunk_height) = Some(height - (chunk_pos.y * CHUNK_SIZE as i32))
                .filter(|h| h >= &1)
                .map(|h| h as usize)
            else {
                continue;
            };
            for y in (0..chunk_height - 1).filter(|h| h < &CHUNK_SIZE) {
                blocks[x][y][z] = Block::Stone;
            }
            if chunk_height - 1 < CHUNK_SIZE {
                blocks[x][chunk_height - 1][z] = Block::Dirt;
            }
            if chunk_height < CHUNK_SIZE {
                blocks[x][chunk_height][z] = Block::Grass;
            }
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

#[derive(Component, PartialEq, Eq, Default)]
struct ChunkPosition(IVec3);

impl ChunkPosition {
    fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition((*p / (CHUNK_SIZE as f32)).floor().as_ivec3())
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
    const CAMERA_HORIZONTAL_BLOCKS_PER_SECOND: f32 = 30.0;
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
