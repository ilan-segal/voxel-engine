use crate::block::{Block, BlockSide};
use crate::chunk::{Chunk, ChunkPosition, CHUNK_SIZE};
use crate::WORLD_LAYER;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::view::RenderLayers;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshGenTasks>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    begin_mesh_gen_tasks,
                    receive_mesh_gen_tasks.after(crate::world::WorldSet),
                ),
            );
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let common_materials = CommonMaterials {
        white: materials.add(Color::WHITE),
    };
    commands.insert_resource(common_materials);
}

#[derive(Resource)]
struct CommonMaterials {
    white: Handle<StandardMaterial>,
}

pub struct MeshTaskData {
    entity: Entity,
    mesh: Mesh,
}

#[derive(Resource, Default)]
pub struct MeshGenTasks(pub HashMap<ChunkPosition, Task<MeshTaskData>>);

fn begin_mesh_gen_tasks(
    mut tasks: ResMut<MeshGenTasks>,
    q_chunk: Query<(Entity, &Chunk, &ChunkPosition), Without<Handle<Mesh>>>,
) {
    for (entity, chunk, pos) in q_chunk.iter() {
        let task_pool = AsyncComputeTaskPool::get();
        if tasks.0.contains_key(pos) {
            continue;
        }
        let cloned_chunk = chunk.clone();
        let task = task_pool.spawn(async move {
            MeshTaskData {
                entity,
                mesh: get_mesh_for_chunk(cloned_chunk),
            }
        });
        tasks.0.insert(*pos, task);
    }
}

fn receive_mesh_gen_tasks(
    mut commands: Commands,
    mut tasks: ResMut<MeshGenTasks>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<CommonMaterials>,
    q_transform: Query<&Transform>,
) {
    tasks.0.retain(|_, task| {
        let Some(data) = block_on(future::poll_once(task)) else {
            return true;
        };
        let e = data.entity;
        if let Some(mut entity) = commands.get_entity(e) {
            entity.insert((
                PbrBundle {
                    mesh: meshes.add(data.mesh),
                    material: materials.white.clone(),
                    transform: *q_transform.get(e).unwrap(),
                    ..default()
                },
                RenderLayers::layer(WORLD_LAYER),
            ));
        };
        return false;
    });
}

#[derive(Debug)]
struct Quad {
    vertices: [Vec3; 4],
    block: Block,
}

fn get_mesh_for_chunk(chunk: Chunk) -> Mesh {
    let mut quads = vec![];
    quads.extend(greedy_mesh(&chunk, BlockSide::Up));
    quads.extend(greedy_mesh(&chunk, BlockSide::Down));
    quads.extend(greedy_mesh(&chunk, BlockSide::North));
    quads.extend(greedy_mesh(&chunk, BlockSide::South));
    quads.extend(greedy_mesh(&chunk, BlockSide::West));
    quads.extend(greedy_mesh(&chunk, BlockSide::East));
    return create_mesh_from_quads(&quads);
}

// TODO: Replace slow implementation with binary mesher
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
            BlockSide::North => [
                Vec3::new(xf + 1.0, yf - 1.0, zf + w),
                Vec3::new(xf + 1.0, yf - 1.0 + h, zf + w),
                Vec3::new(xf + 1.0, yf - 1.0 + h, zf),
                Vec3::new(xf + 1.0, yf - 1.0, zf),
            ],
            BlockSide::South => [
                Vec3::new(xf, yf - 1.0, zf),
                Vec3::new(xf, yf - 1.0 + h, zf),
                Vec3::new(xf, yf - 1.0 + h, zf + w),
                Vec3::new(xf, yf - 1.0, zf + w),
            ],
            BlockSide::West => [
                Vec3::new(xf + h, yf - 1.0, zf),
                Vec3::new(xf + h, yf - 1.0 + w, zf),
                Vec3::new(xf, yf - 1.0 + w, zf),
                Vec3::new(xf, yf - 1.0, zf),
            ],
            BlockSide::East => [
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
        BlockSide::North => (layer, row, col),
        BlockSide::South => (CHUNK_SIZE - 1 - layer, row, col),
        BlockSide::East => (row, col, layer),
        BlockSide::West => (row, col, CHUNK_SIZE - 1 - layer),
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
        .map(|block| block.get_colour().expect("Meshed block should have colour"))
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
