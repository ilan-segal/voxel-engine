use crate::block::Block;
use crate::chunk::{Chunk, CHUNK_SIZE};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_mesh_to_chunks.after(crate::world::WorldSet));
    }
}

// TODO: Non-blocking system
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
