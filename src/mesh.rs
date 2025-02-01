use crate::{
    block::{Block, BlockSide},
    chunk::{
        data::Blocks, layer_to_xyz, position::ChunkPosition, spatial::SpatiallyMapped, Chunk,
        CHUNK_SIZE,
    },
    texture::BlockMaterials,
    world::{
        neighborhood::{ComponentCopy, Neighborhood},
        stage::Stage,
        WorldSet,
    },
    WORLD_LAYER,
};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        view::RenderLayers,
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use itertools::Itertools;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshGenTasks>()
            .add_systems(
                Update,
                (
                    (
                        update_mesh_status,
                        mark_mesh_as_stale,
                        (
                            begin_mesh_gen_tasks,
                            begin_mesh_gen_tasks_for_positionless_chunks,
                        ),
                    )
                        .chain(),
                    receive_mesh_gen_tasks,
                )
                    .after(WorldSet)
                    .in_set(MeshSet),
            )
            .observe(end_mesh_tasks_for_unloaded_chunks);
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct MeshSet;

struct MeshTaskData {
    entity: Entity,
    mesh: Vec<(Block, BlockSide, Mesh)>,
}

#[derive(Component)]
struct Meshed;

#[derive(Component)]
struct CheckedForMesh;

fn update_mesh_status(
    q: Query<(Entity, &Stage), (With<Chunk>, Without<CheckedForMesh>)>,
    mut commands: Commands,
) {
    for (e, stage) in q.iter() {
        if stage == &Stage::final_stage() {
            commands
                .entity(e)
                .remove::<Meshed>()
                .insert(CheckedForMesh);
        } else {
            commands
                .entity(e)
                .insert((CheckedForMesh, Meshed));
        }
    }
}

fn mark_mesh_as_stale(
    mut commands: Commands,
    q_changed_neighborhood: Query<Entity, Changed<Neighborhood<Blocks>>>,
    mut tasks: ResMut<MeshGenTasks>,
) {
    for entity in q_changed_neighborhood.iter() {
        commands
            .entity(entity)
            .remove::<CheckedForMesh>();
        tasks.0.remove(&entity);
    }
}

#[derive(Resource, Default)]
struct MeshGenTasks(HashMap<Entity, Task<MeshTaskData>>);

fn end_mesh_tasks_for_unloaded_chunks(
    trigger: Trigger<OnRemove, Chunk>,
    mut tasks: ResMut<MeshGenTasks>,
) {
    tasks.0.remove(&trigger.entity());
}

fn begin_mesh_gen_tasks(
    mut tasks: ResMut<MeshGenTasks>,
    q_chunk: Query<
        (Entity, &ChunkPosition, &Neighborhood<Blocks>),
        (With<Chunk>, Without<Meshed>, With<CheckedForMesh>),
    >,
    mut commands: Commands,
) {
    for (entity, pos, neighborhood) in q_chunk.iter() {
        let task_pool = AsyncComputeTaskPool::get();
        let Some(middle_chunk) = neighborhood.middle_chunk() else {
            warn!("Chunk at {:?} is absent from index", pos);
            continue;
        };
        commands.entity(entity).insert(Meshed);
        if !middle_chunk.is_meshable() {
            continue;
        }
        let cloned_neighborhood = neighborhood.clone();
        let task = task_pool.spawn(async move {
            MeshTaskData {
                entity,
                mesh: get_meshes_for_chunk(cloned_neighborhood),
            }
        });
        tasks.0.insert(entity, task);
    }
}

fn begin_mesh_gen_tasks_for_positionless_chunks(
    mut tasks: ResMut<MeshGenTasks>,
    q_chunk: Query<
        (Entity, &ComponentCopy<Blocks>),
        (
            With<Chunk>,
            Without<Meshed>,
            With<CheckedForMesh>,
            Without<ChunkPosition>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, blocks) in q_chunk.iter() {
        commands.entity(entity).insert(Meshed);
        let mut neighborhood = Neighborhood::default();
        *neighborhood.get_chunk_mut(0, 0, 0) = Some(blocks.0.clone());
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            MeshTaskData {
                entity,
                mesh: get_meshes_for_chunk(neighborhood),
            }
        });
        tasks.0.insert(entity, task);
    }
}

fn receive_mesh_gen_tasks(
    mut commands: Commands,
    mut tasks: ResMut<MeshGenTasks>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<BlockMaterials>,
    q_render_layers: Query<&RenderLayers>,
) {
    tasks.0.retain(|_, task| {
        let Some(data) = block_on(future::poll_once(task)) else {
            return true;
        };
        let e = data.entity;
        let Some(mut entity) = commands.get_entity(e) else {
            return true;
        };
        entity.despawn_descendants();
        if data.mesh.is_empty() {
            return false;
        }
        let render_layer = q_render_layers
            .get(e)
            .ok()
            .cloned()
            .unwrap_or(RenderLayers::layer(WORLD_LAYER));
        entity.with_children(|builder| {
            for (block, side, mesh) in data.mesh {
                builder.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: materials
                            .get(&block, &side)
                            .unwrap()
                            .clone_weak(),
                        ..default()
                    },
                    render_layer.clone(),
                ));
            }
        });
        return false;
    });
}

#[derive(Debug, Clone)]
struct Quad {
    block: Block,
    side: BlockSide,
    vertices: [Vec3; 4],
    ao_factors: [u8; 4],
    uvs: [[f32; 2]; 4],
}

impl Quad {
    fn rotate_against_anisotropy(&mut self) {
        if self.ao_factors[0] + self.ao_factors[2] > self.ao_factors[1] + self.ao_factors[3] {
            self.rotate_left(1);
        }
    }

    fn rotate_left(&mut self, mid: usize) {
        self.vertices.rotate_left(mid);
        self.ao_factors.rotate_left(mid);
        self.uvs.rotate_left(mid);
    }
}

fn get_meshes_for_chunk(chunk: Neighborhood<Blocks>) -> Vec<(Block, BlockSide, Mesh)> {
    let mut quads = vec![];
    quads.extend(greedy_mesh(&chunk, BlockSide::Up));
    quads.extend(greedy_mesh(&chunk, BlockSide::Down));
    quads.extend(greedy_mesh(&chunk, BlockSide::North));
    quads.extend(greedy_mesh(&chunk, BlockSide::South));
    quads.extend(greedy_mesh(&chunk, BlockSide::West));
    quads.extend(greedy_mesh(&chunk, BlockSide::East));
    return create_meshes(quads);
}

// TODO: Replace slow implementation with binary mesher
fn greedy_mesh(chunk: &Neighborhood<Blocks>, direction: BlockSide) -> Vec<Quad> {
    let mut quads: Vec<Quad> = vec![];
    let middle = chunk
        .middle_chunk()
        .clone()
        .expect("Already checked");
    let mut blocks = middle.as_ref().clone();
    for layer in 0..CHUNK_SIZE {
        for row in 0..CHUNK_SIZE {
            for col in 0..CHUNK_SIZE {
                let block = blocks.get_from_layer_coords(&direction, layer, row, col);
                if block == &Block::Air
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
                let vertices = get_quad_corners(&direction, layer, row, height, col, width);
                let ao_factors = if direction == BlockSide::Down {
                    [
                        bottom_right_ao_factor,
                        bottom_left_ao_factor,
                        top_left_ao_factor,
                        top_right_ao_factor,
                    ]
                } else {
                    [
                        bottom_left_ao_factor,
                        bottom_right_ao_factor,
                        top_right_ao_factor,
                        top_left_ao_factor,
                    ]
                };
                let u = width as f32 + 1.0;
                let v = height as f32 + 1.0;
                let uvs = if direction == BlockSide::North || direction == BlockSide::West {
                    [[v, u], [v, 0.0], [0., 0.], [0., u]]
                } else {
                    [[0., v], [u, v], [u, 0.0], [0., 0.]]
                };

                let quad = Quad {
                    side: direction,
                    vertices,
                    ao_factors,
                    block: *block,
                    uvs,
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
    chunk: &Neighborhood<Blocks>,
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
    ) -> &Block;

    fn clear_at(&mut self, direction: &BlockSide, layer: usize, row: usize, col: usize);
}

impl LayerIndexable for Blocks {
    fn get_from_layer_coords(
        &self,
        direction: &BlockSide,
        layer: usize,
        row: usize,
        col: usize,
    ) -> &Block {
        let (x, y, z) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
        self.at_pos([x as usize, y as usize, z as usize])
    }

    fn clear_at(&mut self, direction: &BlockSide, layer: usize, row: usize, col: usize) {
        let (x, y, z) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
        *self.at_pos_mut([x as usize, y as usize, z as usize]) = Block::Air;
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
            Vec3::new(xf, yf - 1.0, zf + w),
            Vec3::new(xf, yf - 1.0, zf),
            Vec3::new(xf + h, yf - 1.0, zf),
            Vec3::new(xf + h, yf - 1.0, zf + w),
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

fn create_meshes(quads: Vec<Quad>) -> Vec<(Block, BlockSide, Mesh)> {
    quads
        .iter()
        .map(|q| (q.block, optimize_side_choice(&q.block, &q.side), q))
        .sorted_by_key(|(block, side, _)| (*block, *side))
        .chunk_by(|(block, side, _)| (*block, *side))
        .into_iter()
        .filter_map(|((block, side), qs)| {
            let mesh = create_mesh_from_quads(qs.map(|qs| qs.2).cloned().collect())?;
            Some((block, side, mesh))
        })
        .collect()
}

fn optimize_side_choice(block: &Block, side: &BlockSide) -> BlockSide {
    // Normalize so there are fewer separate meshes
    match (block, side) {
        (Block::Wood, BlockSide::Up) | (Block::Wood, BlockSide::Down) => BlockSide::Up,
        (Block::Wood, _) => BlockSide::East,
        _ => BlockSide::default(),
    }
}

fn create_mesh_from_quads(mut quads: Vec<Quad>) -> Option<Mesh> {
    if quads.is_empty() {
        return None;
    }
    for i in 0..quads.len() {
        quads[i].rotate_against_anisotropy();
    }
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
            let quad_index = quad_index as u32;
            [
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
        .collect::<Vec<_>>();
    // let colour = block.get_colour().unwrap_or_default();
    let colours = quads
        .iter()
        .flat_map(|q| {
            q.ao_factors.iter().map(move |factor| {
                let lum = 0.6_f32.powi((*factor).into());
                return Color::WHITE.with_luminance(lum);
            })
        })
        .map(|c| c.to_linear().to_f32_array())
        .collect::<Vec<_>>();
    let uv = quads
        .iter()
        .flat_map(|q| q.uvs)
        .collect::<Vec<_>>();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_indices(Indices::U32(indices));
    return Some(mesh);
}
