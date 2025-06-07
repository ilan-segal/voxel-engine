use bevy::{
    ecs::entity::EntityHashMap,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        view::RenderLayers,
    },
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};

use crate::{
    block::{Block, BlockSide},
    chunk::{
        data::Blocks, layer_to_xyz, position::ChunkPosition, spatial::SpatiallyMapped, Chunk,
        CHUNK_SIZE,
    },
    material::ATTRIBUTE_TERRAIN_VERTEX_DATA,
    texture::BlockMaterials,
    world::{
        neighborhood::{ComponentCopy, Neighborhood},
        stage::Stage,
        WorldSet,
    },
    WORLD_LAYER,
};

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
            .add_observer(end_mesh_tasks_for_unloaded_chunks);
    }
}

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct MeshSet;

struct MeshTaskData {
    entity: Entity,
    mesh: Option<Mesh>,
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
            commands.entity(e).remove::<Meshed>().insert(CheckedForMesh);
        } else {
            commands.entity(e).insert((CheckedForMesh, Meshed));
        }
    }
}

fn mark_mesh_as_stale(
    mut commands: Commands,
    q_changed_neighborhood: Query<Entity, Changed<Neighborhood<Blocks>>>,
    mut tasks: ResMut<MeshGenTasks>,
) {
    for entity in q_changed_neighborhood.iter() {
        commands.entity(entity).remove::<CheckedForMesh>();
        tasks.0.remove(&entity);
    }
}

#[derive(Resource, Default)]
struct MeshGenTasks(EntityHashMap<Task<MeshTaskData>>);

fn end_mesh_tasks_for_unloaded_chunks(
    trigger: Trigger<OnRemove, Chunk>,
    mut tasks: ResMut<MeshGenTasks>,
) {
    tasks.0.remove(&trigger.target());
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
                mesh: chunk_mesh(cloned_neighborhood),
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
                mesh: chunk_mesh(neighborhood),
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
        let Ok(mut entity) = commands.get_entity(e) else {
            return true;
        };
        entity.despawn_related::<Children>();
        let Some(mesh) = data.mesh else {
            return false;
        };
        let render_layer = q_render_layers
            .get(e)
            .ok()
            .cloned()
            .unwrap_or(RenderLayers::layer(WORLD_LAYER));
        entity.with_children(|builder| {
            builder.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.terrain.clone_weak()),
                render_layer.clone(),
            ));
        });
        return false;
    });
}

#[derive(Debug, Clone)]
struct Quad {
    // block: Block,
    side: BlockSide,
    vertices: [IVec3; 4],
    ao_factors: [u8; 4],
    // uvs: [[f32; 2]; 4],
}

/*

00000 <- 0
10000 <- 32

*/

impl Quad {
    fn rotate_against_anisotropy(&mut self) {
        if self.ao_factors[0] + self.ao_factors[2] > self.ao_factors[1] + self.ao_factors[3] {
            self.rotate_left(1);
        }
    }

    fn rotate_left(&mut self, mid: usize) {
        self.vertices.rotate_left(mid);
        self.ao_factors.rotate_left(mid);
        // self.uvs.rotate_left(mid);
    }

    fn get_vertex_data(&self) -> [u32; 4] {
        std::array::from_fn(|idx| self.get_single_vertex_data(idx))
    }

    fn get_single_vertex_data(&self, i: usize) -> u32 {
        let normal_index: u32 = match self.side {
            BlockSide::North => 0,
            BlockSide::South => 1,
            BlockSide::Up => 2,
            BlockSide::Down => 3,
            BlockSide::East => 4,
            BlockSide::West => 5,
        };
        let xs = self.vertices[i].to_array();
        let [local_x, local_y, local_z] = xs.map(|x| u32::try_from(x).unwrap());
        let ao_factor = self.ao_factors[i] as u32;
        let block_id = 0;
        return local_x
            | (local_y << 6)
            | (local_z << 12)
            | (normal_index << 18)
            | (ao_factor << 21)
            | (block_id << 23);
    }
}

fn chunk_mesh(chunk: Neighborhood<Blocks>) -> Option<Mesh> {
    let mut quads = vec![];
    quads.extend(greedy_mesh(&chunk, BlockSide::Up));
    quads.extend(greedy_mesh(&chunk, BlockSide::Down));
    quads.extend(greedy_mesh(&chunk, BlockSide::North));
    quads.extend(greedy_mesh(&chunk, BlockSide::South));
    quads.extend(greedy_mesh(&chunk, BlockSide::West));
    quads.extend(greedy_mesh(&chunk, BlockSide::East));
    return create_mesh_from_quads(quads);
}

// TODO: Replace slow implementation with binary mesher
fn greedy_mesh(chunk: &Neighborhood<Blocks>, direction: BlockSide) -> Vec<Quad> {
    let mut quads: Vec<Quad> = vec![];
    let middle = chunk.middle_chunk().clone().expect("Already checked");
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

                let quad = Quad {
                    side: direction,
                    vertices,
                    ao_factors,
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
) -> [IVec3; 4] {
    let (xf, yf, zf) = layer_to_xyz(direction, layer as i32, row as i32, col as i32);
    let w = width as i32 + 1;
    let h = height as i32 + 1;
    match direction {
        BlockSide::Up => [
            IVec3::new(xf, yf, zf),
            IVec3::new(xf, yf, zf + w),
            IVec3::new(xf + h, yf, zf + w),
            IVec3::new(xf + h, yf, zf),
        ],
        BlockSide::Down => [
            IVec3::new(xf, yf, zf + w),
            IVec3::new(xf, yf, zf),
            IVec3::new(xf + h, yf, zf),
            IVec3::new(xf + h, yf, zf + w),
        ],
        BlockSide::North => [
            IVec3::new(xf + 1, yf, zf),
            IVec3::new(xf + 1, yf + w, zf),
            IVec3::new(xf + 1, yf + w, zf + h),
            IVec3::new(xf + 1, yf, zf + h),
        ],
        BlockSide::South => [
            IVec3::new(xf, yf, zf),
            IVec3::new(xf, yf, zf + w),
            IVec3::new(xf, yf + h, zf + w),
            IVec3::new(xf, yf + h, zf),
        ],
        BlockSide::West => [
            IVec3::new(xf, yf, zf),
            IVec3::new(xf, yf + w, zf),
            IVec3::new(xf + h, yf + w, zf),
            IVec3::new(xf + h, yf, zf),
        ],
        BlockSide::East => [
            IVec3::new(xf, yf, zf + 1),
            IVec3::new(xf + w, yf, zf + 1),
            IVec3::new(xf + w, yf + h, zf + 1),
            IVec3::new(xf, yf + h, zf + 1),
        ],
    }
}

fn create_mesh_from_quads(mut quads: Vec<Quad>) -> Option<Mesh> {
    if quads.is_empty() {
        return None;
    }
    for i in 0..quads.len() {
        quads[i].rotate_against_anisotropy();
    }
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
    let vertex_data = quads
        .iter()
        .flat_map(|q| q.get_vertex_data())
        .collect::<Vec<_>>();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(ATTRIBUTE_TERRAIN_VERTEX_DATA, vertex_data)
    .with_inserted_indices(Indices::U32(indices));
    return Some(mesh);
}
