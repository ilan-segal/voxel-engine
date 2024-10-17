use crate::block::{Block, BlockSide};
use crate::chunk::{layer_to_xyz, Chunk, ChunkIndex, ChunkNeighborhood, ChunkPosition, CHUNK_SIZE};
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
        white: materials.add(StandardMaterial {
            perceptual_roughness: 1.0,
            base_color: Color::WHITE,
            ..Default::default()
        }),
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
    q_chunk: Query<(Entity, &ChunkPosition), (With<Chunk>, Without<Handle<Mesh>>)>,
    chunk_index: Res<ChunkIndex>,
) {
    for (entity, pos) in q_chunk.iter() {
        let task_pool = AsyncComputeTaskPool::get();
        if tasks.0.contains_key(pos) {
            continue;
        }
        let cloned_chunk = chunk_index.get_neighborhood(&pos.0);
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
    ao_factors: [u8; 4],
    block: Block,
}

fn get_mesh_for_chunk(chunk: ChunkNeighborhood) -> Mesh {
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
fn greedy_mesh(chunk: &ChunkNeighborhood, direction: BlockSide) -> Vec<Quad> {
    let mut quads: Vec<Quad> = vec![];
    let mut blocks = *chunk.middle();
    for layer in 0..CHUNK_SIZE {
        for row in 0..CHUNK_SIZE {
            for col in 0..CHUNK_SIZE {
                let block = blocks.get_from_layer_coords(&direction, layer, row, col);
                if block == Block::Air
                    || chunk.block_is_hidden_from_above(
                        &direction,
                        layer as i32,
                        row as i32,
                        col as i32,
                    )
                {
                    continue;
                }
                let bottom_left_ao_factor =
                    get_ao_factor(chunk, &direction, layer, row, col, AoCorner::BottomLeft);
                let top_left_ao_factor =
                    get_ao_factor(chunk, &direction, layer, row, col, AoCorner::TopLeft);
                let bottom_right_ao_factor =
                    get_ao_factor(chunk, &direction, layer, row, col, AoCorner::BottomRight);
                let top_right_ao_factor =
                    get_ao_factor(chunk, &direction, layer, row, col, AoCorner::TopRight);
                let mut height = 0;
                if bottom_left_ao_factor == top_left_ao_factor
                    && bottom_right_ao_factor == top_right_ao_factor
                {
                    while height + row < CHUNK_SIZE - 1
                        && block
                            == blocks.get_from_layer_coords(
                                &direction,
                                layer,
                                height + row + 1,
                                col,
                            )
                        && !chunk.block_is_hidden_from_above(
                            &direction,
                            layer as i32,
                            (height + row + 1) as i32,
                            col as i32,
                        )
                    {
                        let new_top_left_factor = get_ao_factor(
                            chunk,
                            &direction,
                            layer,
                            row + height + 1,
                            col,
                            AoCorner::TopLeft,
                        );
                        if new_top_left_factor != top_left_ao_factor {
                            break;
                        }
                        let new_top_right_factor = get_ao_factor(
                            chunk,
                            &direction,
                            layer,
                            row + height + 1,
                            col,
                            AoCorner::TopRight,
                        );
                        if new_top_right_factor != top_right_ao_factor {
                            break;
                        }

                        height += 1;
                    }
                }
                let mut width = 0;
                if bottom_left_ao_factor == bottom_right_ao_factor
                    && top_left_ao_factor == top_right_ao_factor
                {
                    while col + width < CHUNK_SIZE - 1
                        && (row..=height + row).all(|cur_row| {
                            block
                                == blocks.get_from_layer_coords(
                                    &direction,
                                    layer,
                                    cur_row,
                                    col + width + 1,
                                )
                                && !chunk.block_is_hidden_from_above(
                                    &direction,
                                    layer as i32,
                                    cur_row as i32,
                                    (col + width) as i32 + 1,
                                )
                        })
                    {
                        let new_bottom_right_factor = get_ao_factor(
                            chunk,
                            &direction,
                            layer,
                            row + height,
                            col + width + 1,
                            AoCorner::BottomRight,
                        );
                        if new_bottom_right_factor != bottom_right_ao_factor {
                            break;
                        }
                        let new_top_right_factor = get_ao_factor(
                            chunk,
                            &direction,
                            layer,
                            row + height,
                            col + width + 1,
                            AoCorner::TopRight,
                        );
                        if new_top_right_factor != top_right_ao_factor {
                            break;
                        }
                        width += 1;
                    }
                }
                // TODO rotate vertices if necessary, to prevent anisotropy
                let vertices = get_quad_corners(&direction, layer, row, height, col, width);
                let ao_factors = [
                    bottom_left_ao_factor,
                    bottom_right_ao_factor,
                    top_right_ao_factor,
                    top_left_ao_factor,
                ];
                let quad = Quad {
                    vertices,
                    ao_factors,
                    block,
                };
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

fn get_ao_factor(
    chunk: &ChunkNeighborhood,
    side: &BlockSide,
    layer: usize,
    row: usize,
    col: usize,
    corner: AoCorner,
) -> u8 {
    let layer = layer as i32;
    let row = row as i32;
    let col = col as i32;
    let ((left_row, left_col), (right_row, right_col), (corner_row, corner_col)) = match corner {
        AoCorner::BottomLeft => ((row - 1, col), (row, col - 1), (row - 1, col - 1)),
        AoCorner::BottomRight => ((row - 1, col), (row, col + 1), (row - 1, col + 1)),
        AoCorner::TopLeft => ((row + 1, col), (row, col - 1), (row + 1, col - 1)),
        AoCorner::TopRight => ((row + 1, col), (row, col + 1), (row + 1, col + 1)),
    };
    let left_block = chunk.count_block(side, layer + 1, left_row, left_col);
    let right_block = chunk.count_block(side, layer + 1, right_row, right_col);
    if left_block != 0 && right_block != 0 {
        return 3;
    } else {
        return left_block
            + right_block
            + chunk.count_block(side, layer + 1, corner_row, corner_col);
    }
}

enum AoCorner {
    BottomLeft,
    TopLeft,
    BottomRight,
    TopRight,
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
}

impl LayerIndexable for [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] {
    fn get_from_layer_coords(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        col: usize,
    ) -> Block {
        let (x, y, z) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
        self[x as usize][y as usize][z as usize]
    }

    fn clear_at(&mut self, direction: &BlockSide, layer: usize, row: usize, col: usize) {
        let (x, y, z) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
        self[x as usize][y as usize][z as usize] = Block::Air;
    }
}

fn get_quad_corners(
    direction: &BlockSide,
    layer: usize,
    row: usize,
    height: usize,
    col: usize,
    width: usize,
) -> [Vec3; 4] {
    let (x, y, z) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
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
            Vec3::new(xf, yf, zf + w),
            Vec3::new(xf + h, yf, zf + w),
            Vec3::new(xf + h, yf, zf),
        ],
        BlockSide::Down => [
            Vec3::new(xf, yf - 1.0, zf),
            Vec3::new(xf + h, yf - 1.0, zf),
            Vec3::new(xf + h, yf - 1.0, zf + w),
            Vec3::new(xf, yf - 1.0, zf + w),
        ],
        BlockSide::North => [
            Vec3::new(xf + 1.0, yf - 1.0, zf),
            Vec3::new(xf + 1.0, yf - 1.0 + w, zf),
            Vec3::new(xf + 1.0, yf - 1.0 + w, zf + h),
            Vec3::new(xf + 1.0, yf - 1.0, zf + h),
        ],
        BlockSide::South => [
            Vec3::new(xf, yf - 1.0, zf),
            Vec3::new(xf, yf - 1.0, zf + w),
            Vec3::new(xf, yf - 1.0 + h, zf + w),
            Vec3::new(xf, yf - 1.0 + h, zf),
        ],
        BlockSide::West => [
            Vec3::new(xf, yf - 1.0, zf),
            Vec3::new(xf, yf - 1.0 + w, zf),
            Vec3::new(xf + h, yf - 1.0 + w, zf),
            Vec3::new(xf + h, yf - 1.0, zf),
        ],
        BlockSide::East => [
            Vec3::new(xf, yf - 1.0, zf + 1.0),
            Vec3::new(xf + w, yf - 1.0, zf + 1.0),
            Vec3::new(xf + w, yf - 1.0 + h, zf + 1.0),
            Vec3::new(xf, yf - 1.0 + h, zf + 1.0),
        ],
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
            return a.cross(b).normalize();
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
                4 * quad_index + 0,
                4 * quad_index + 1,
                4 * quad_index + 2,
                // Triangle b
                4 * quad_index + 0,
                4 * quad_index + 2,
                4 * quad_index + 3,
            ]
        })
        .map(|idx| idx as u32)
        .collect::<Vec<_>>();
    let colours = quads
        .iter()
        .flat_map(|q: &Quad| {
            let colour = q
                .block
                .get_colour()
                .expect("Meshed block should have colour");

            return q.ao_factors.iter().map(move |factor| {
                let lum = 0.6_f32.powi((*factor).into());
                return colour.with_luminance(lum);
            });
        })
        .map(|c| c.to_linear().to_f32_array())
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
