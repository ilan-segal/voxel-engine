use bevy::{
    color::palettes::{
        basic::{GREEN, SILVER},
        css::BROWN,
    },
    input::mouse::MouseMotion,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
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
        .init_resource::<WorldGenNoise>()
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_camera,
                toggle_wireframe,
                update_camera_chunk_position,
                (update_loaded_chunks, add_mesh_to_chunks).chain(),
            ),
        )
        .run();
}

#[derive(Debug)]
struct Quad {
    vertices: [Vec3; 4],
    block: Block,
}

enum BlockSide {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

fn get_mesh_for_chunk(chunk: &Chunk) -> Mesh {
    let mut quads = vec![];
    quads.extend(greedy_mesh(chunk, BlockSide::Up));
    quads.extend(greedy_mesh(chunk, BlockSide::Down));
    quads.extend(greedy_mesh(chunk, BlockSide::Left));
    quads.extend(greedy_mesh(chunk, BlockSide::Right));
    quads.extend(greedy_mesh(chunk, BlockSide::Front));
    quads.extend(greedy_mesh(chunk, BlockSide::Back));
    return create_mesh_from_quads(&quads);
}

fn greedy_mesh(chunk: &Chunk, direction: BlockSide) -> Vec<Quad> {
    let mut quads: Vec<Quad> = vec![];
    let mut blocks = chunk.blocks;
    for layer in 0..CHUNK_SIZE {
        for row in 0..CHUNK_SIZE {
            for col in 0..CHUNK_SIZE {
                let block_is_hidden_from_above = |row: usize, col: usize, layer: usize| {
                    layer < CHUNK_SIZE - 1
                        && blocks.get_from_layer_coords(&direction, layer + 1, row, col)
                            != Block::Air
                };
                let block = blocks.get_from_layer_coords(&direction, layer, row, col);
                if block == Block::Air || block_is_hidden_from_above(row, col, layer) {
                    continue;
                }
                let mut height = 0;
                let mut width = 0;
                while height + row < CHUNK_SIZE - 1
                    && block
                        == blocks.get_from_layer_coords(
                            &direction,
                            layer,
                            height + row + 1,
                            col + width,
                        )
                {
                    height += 1;
                }
                while col + width < CHUNK_SIZE - 1
                    && (row..=height + row).all(|cur_row| {
                        block
                            == blocks.get_from_layer_coords(
                                &direction,
                                layer,
                                cur_row,
                                col + width + 1,
                            )
                            && !block_is_hidden_from_above(cur_row, col + width + 1, layer)
                    })
                {
                    width += 1;
                }
                let vertices = blocks.get_quad_corners(&direction, layer, row, height, col, width);
                let quad = Quad { vertices, block };
                quads.push(quad);
                for cur_row in row..=height + row {
                    for cur_col in col..=col + width {
                        blocks.clear_at(&direction, layer, cur_row, cur_col);
                    }
                }
            }
        }
    }
    return quads;
}

trait LayerIndexable {
    fn get_from_layer_coords(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        col: usize,
    ) -> Block;

    fn clear_at(&mut self, direction: &BlockSide, layer: usize, row: usize, col: usize);

    fn get_quad_corners(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        height: usize,
        col: usize,
        width: usize,
    ) -> [Vec3; 4];
}

impl LayerIndexable for [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] {
    fn get_from_layer_coords(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        col: usize,
    ) -> Block {
        let (x, y, z) = get_xyz_from_layer_indices(direction, layer, row, col);
        self[x][y][z]
    }

    fn clear_at(&mut self, direction: &BlockSide, layer: usize, row: usize, col: usize) {
        let (x, y, z) = get_xyz_from_layer_indices(direction, layer, row, col);
        self[x][y][z] = Block::Air;
    }

    fn get_quad_corners(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        height: usize,
        col: usize,
        width: usize,
    ) -> [Vec3; 4] {
        let (x, y, z) = get_xyz_from_layer_indices(direction, layer, row, col);
        let (xf, yf, zf, h, w) = (
            x as f32,
            y as f32,
            z as f32,
            height as f32 + 1.0,
            width as f32 + 1.0,
        );
        match direction {
            BlockSide::Up => [
                Vec3::new(xf, yf, zf),
                Vec3::new(xf + h, yf, zf),
                Vec3::new(xf + h, yf, zf + w),
                Vec3::new(xf, yf, zf + w),
            ],
            BlockSide::Down => [
                Vec3::new(xf, yf - 1.0, zf + w),
                Vec3::new(xf + h, yf - 1.0, zf + w),
                Vec3::new(xf + h, yf - 1.0, zf),
                Vec3::new(xf, yf - 1.0, zf),
            ],
            BlockSide::Left => [
                Vec3::new(xf + 1.0, yf - 1.0, zf + w),
                Vec3::new(xf + 1.0, yf - 1.0 + h, zf + w),
                Vec3::new(xf + 1.0, yf - 1.0 + h, zf),
                Vec3::new(xf + 1.0, yf - 1.0, zf),
            ],
            BlockSide::Right => [
                Vec3::new(xf, yf - 1.0, zf),
                Vec3::new(xf, yf - 1.0 + h, zf),
                Vec3::new(xf, yf - 1.0 + h, zf + w),
                Vec3::new(xf, yf - 1.0, zf + w),
            ],
            BlockSide::Front => [
                Vec3::new(xf + h, yf - 1.0, zf),
                Vec3::new(xf + h, yf - 1.0 + w, zf),
                Vec3::new(xf, yf - 1.0 + w, zf),
                Vec3::new(xf, yf - 1.0, zf),
            ],
            BlockSide::Back => [
                Vec3::new(xf, yf - 1.0, zf + 1.0),
                Vec3::new(xf, yf - 1.0 + w, zf + 1.0),
                Vec3::new(xf + h, yf - 1.0 + w, zf + 1.0),
                Vec3::new(xf + h, yf - 1.0, zf + 1.0),
            ],
        }
    }
}

fn get_xyz_from_layer_indices(
    direction: &BlockSide,
    layer: usize,
    row: usize,
    col: usize,
) -> (usize, usize, usize) {
    match direction {
        BlockSide::Up => (row, layer, col),
        BlockSide::Down => (row, CHUNK_SIZE - 1 - layer, col),
        BlockSide::Left => (layer, row, col),
        BlockSide::Right => (CHUNK_SIZE - 1 - layer, row, col),
        BlockSide::Back => (row, col, layer),
        BlockSide::Front => (row, col, CHUNK_SIZE - 1 - layer),
    }
}

fn add_mesh_to_chunks(
    mut commands: Commands,
    q_chunk: Query<(Entity, &Transform, &Chunk), Without<Handle<Mesh>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, transform, chunk) in q_chunk.iter() {
        // let quads = greedy_mesher_y(chunk);
        // let mesh = create_mesh_from_quads(&quads);
        let mesh = get_mesh_for_chunk(chunk);
        if let Some(mut entity) = commands.get_entity(e) {
            entity.insert(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::WHITE),
                transform: *transform,
                // transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            });
        }
    }
}

fn create_mesh_from_quads(quads: &Vec<Quad>) -> Mesh {
    let vertices = quads
        .iter()
        .flat_map(|q| q.vertices.iter())
        .map(|v| v.to_array())
        .collect::<Vec<_>>();
    let normals = quads
        .iter()
        .map(|q| q.vertices)
        .map(|vs| {
            let a = vs[1] - vs[0];
            let b = vs[2] - vs[0];
            return a.cross(b).normalize() * -1.0;
        })
        .map(|norm| norm.to_array())
        .flat_map(|norm| std::iter::repeat_n(norm, 4))
        .collect::<Vec<_>>();
    let indices = (0..quads.len())
        .flat_map(|quad_index| {
            vec![
                /*
                3---2
                |b /|
                | / |
                |/ a|
                0---1
                 */
                // Triangle a
                4 * quad_index + 2,
                4 * quad_index + 1,
                4 * quad_index + 0,
                // Triangle b
                4 * quad_index + 3,
                4 * quad_index + 2,
                4 * quad_index + 0,
            ]
        })
        .map(|idx| idx as u32)
        .collect::<Vec<_>>();
    let colours = quads
        .iter()
        .map(|q| q.block)
        .map(|block| {
            block
                .get_colour()
                .expect("Meshed block should have colour")
        })
        .map(|c| c.to_linear().to_f32_array())
        .flat_map(|m| std::iter::repeat_n(m, 4))
        .collect::<Vec<_>>();
    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

struct NoiseGenerator {
    perlin: Perlin,
    scale: f64,
    amplitude: f64,
    offset: f64,
}

impl NoiseGenerator {
    fn sample(&self, x: f64, y: f64) -> f64 {
        let sample_x = x / self.scale + self.offset;
        let sample_y = y / self.scale + self.offset;
        return self.perlin.get([sample_x, sample_y]) * self.amplitude;
    }
}

#[derive(Resource)]
struct WorldGenNoise(Vec<NoiseGenerator>);

impl WorldGenNoise {
    // Returns in range [0, 1]
    fn sample(&self, x: i32, y: i32) -> f64 {
        let mut total_sample = 0.;
        let mut total_amplitude = 0.;
        for g in &self.0 {
            total_sample += g.sample(x as f64, y as f64);
            total_amplitude += g.amplitude;
        }
        total_sample /= total_amplitude;
        0.5 + 0.5 * total_sample
    }
}

impl Default for WorldGenNoise {
    fn default() -> Self {
        Self(vec![
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 100.0,
                amplitude: 1.0,
                offset: 0.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 50.0,
                amplitude: 0.5,
                offset: 10.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(WORLD_SEED),
                scale: 25.0,
                amplitude: 0.25,
                offset: 20.0,
            },
        ])
    }
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
    world_gen_noise: Res<WorldGenNoise>,
) {
    let Ok(pos) = q_camera_position.get_single() else {
        return;
    };
    let camera_position = pos.compute_transform().translation;
    let chunk_pos = ChunkPosition::from_world_position(&camera_position);
    // Determine position of chunks that should be loaded
    let mut should_be_loaded_positions: HashSet<IVec3> = HashSet::new();
    const LOAD_DISTANCE_CHUNKS: i32 = 4;
    for chunk_x in -LOAD_DISTANCE_CHUNKS..=LOAD_DISTANCE_CHUNKS {
        for chunk_y in 0..2 {
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
            // The chunk should be unloaded since it's not in our set
            commands
                .entity(entity)
                .despawn_recursive();
        }
    }
    // Finally, load the new chunks
    for pos in should_be_loaded_positions {
        let chunk = generate_chunk(&world_gen_noise, &pos);
        commands.spawn((
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
        ));
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

fn generate_chunk(noise: &WorldGenNoise, chunk_pos: &IVec3) -> Chunk {
    const SCALE: f64 = 60.0;
    let mut blocks = default::<[[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>();
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let height = (noise.sample(
                x as i32 + chunk_pos.x * CHUNK_SIZE as i32,
                z as i32 + chunk_pos.z * CHUNK_SIZE as i32,
            ) * SCALE) as i32
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

#[derive(Component, PartialEq, Eq, Default)]
struct ChunkPosition(IVec3);

impl ChunkPosition {
    fn from_world_position(p: &Vec3) -> Self {
        ChunkPosition(
            (*p / (CHUNK_SIZE as f32))
                .floor()
                .as_ivec3(),
        )
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
}

impl Block {
    fn get_colour(&self) -> Option<Color> {
        match self {
            Self::Stone => Some(SILVER),
            Self::Grass => Some(GREEN),
            Self::Dirt => Some(BROWN),
            _ => None,
        }
        .map(Color::from)
    }
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
    const CAMERA_VERTICAL_BLOCKS_PER_SECOND: f32 = 30.0;
    const CAMERA_HORIZONTAL_BLOCKS_PER_SECOND: f32 = 600.0;
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
            let (yaw, _, _) = transform
                .rotation
                .to_euler(EulerRot::YXZ);
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
            let (yaw, mut pitch, _) = transform
                .rotation
                .to_euler(EulerRot::YXZ);
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
