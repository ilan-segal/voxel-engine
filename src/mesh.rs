use crate::block::{Block, BlockSide};
use crate::chunk::{BitWiseOps, Chunk, ChunkMask, ChunkPosition, LayerMask, CHUNK_SIZE};
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

fn get_mesh_for_chunk(chunk: Chunk) -> Mesh {
    let mut quads = vec![];
    quads.extend(get_quads_for_side(&chunk, BlockSide::Up));
    // quads.extend(greedy_mesh(&chunk, BlockSide::Up));
    // quads.extend(greedy_mesh(&chunk, BlockSide::Down));
    // quads.extend(greedy_mesh(&chunk, BlockSide::North));
    // quads.extend(greedy_mesh(&chunk, BlockSide::South));
    // quads.extend(greedy_mesh(&chunk, BlockSide::West));
    // quads.extend(greedy_mesh(&chunk, BlockSide::East));
    return greedy_quads_to_mesh(&quads);
}

struct Quad {
    vertices: [Vec3; 4],
    block: Block,
}

struct GreedyQuad {
    row: u32,
    col: u32,
    h: u32,
    w: u32,
}

fn get_quads_for_side(chunk: &Chunk, side: BlockSide) -> Vec<Quad> {
    let mut quads = vec![];
    let empty_layer = LayerMask::default();
    let visibility = chunk
        .get_opacity_mask()
        .iter()
        .chain(std::iter::once(&empty_layer))
        .map_windows(|[la, lb]| std::array::from_fn(|i| la[i] & !lb[i]))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    for (block, mask) in chunk.masks.iter() {
        if block == &Block::Air {
            continue;
        }
        let cur_quads = greedy_quad_vertices_for_side(mask, &visibility, &side)
            .iter()
            .map(|vertices| Quad {
                vertices: *vertices,
                block: *block,
            })
            .collect::<Vec<_>>();
        quads.extend(cur_quads);
    }
    return quads;
}

fn greedy_quad_vertices_for_side(
    chunk: &ChunkMask,
    visibility: &ChunkMask,
    side: &BlockSide,
) -> Vec<[Vec3; 4]> {
    let mut vertices = vec![];

    for (layer_idx, layer) in chunk
        .iter()
        .zip(visibility)
        .map(|(layer, vis)| vis.and(layer))
        .enumerate()
    {
        let corners = greedy_quads_for_layer(layer)
            .iter()
            .map(|q| {
                get_quad_corners(
                    side,
                    layer_idx,
                    q.row as usize,
                    q.col as usize,
                    q.h as usize,
                    q.w as usize,
                )
            })
            .collect::<Vec<_>>();
        vertices.extend(corners);
    }
    return vertices;
}

fn greedy_quads_for_layer(mut layer: LayerMask) -> Vec<GreedyQuad> {
    let mut quads = vec![];
    for row in 0..layer.len() {
        let mut y = 0;
        while y < CHUNK_SIZE as u32 {
            y += (layer[row] >> y).trailing_zeros();
            if y >= CHUNK_SIZE as u32 {
                continue;
            }
            let h = (layer[row] >> y).trailing_ones();
            let h_as_mask = u32::checked_shl(1, h).map_or(!0, |v| v - 1);
            let mask = h_as_mask << y;
            let mut w = 1;
            while row + w < CHUNK_SIZE {
                let masked_next_row = layer[row + w] & mask;
                if masked_next_row != mask {
                    break;
                }
                layer[row + w] &= !mask;
                w += 1;
            }
            quads.push(GreedyQuad {
                col: row as u32,
                row: y,
                h: h - 1,
                w: w as u32 - 1,
            });
            y += h;
        }
    }
    return quads;
}

fn get_quad_corners(
    direction: &BlockSide,
    layer: usize,
    row: usize,
    col: usize,
    height: usize,
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

fn greedy_quads_to_mesh(quads: &Vec<Quad>) -> Mesh {
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
