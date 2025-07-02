use std::collections::VecDeque;

use crate::{
    block::Block,
    chunk::{
        data::Blocks, position::ChunkPosition, spatial::SpatiallyMapped, CHUNK_SIZE, CHUNK_SIZE_I32,
    },
    physics::{
        aabb::Aabb,
        collision::{Collidable, Collision},
        gravity::Gravity,
        PhysicsSystemSet,
    },
    ui::block_icons::BlockMeshes,
    world::{
        index::ChunkIndex,
        neighborhood::{ComponentIndex, Neighborhood},
        stage::Stage,
        WorldSet,
    },
};
use bevy::{ecs::query::QueryData, prelude::*};

mod dirt;
mod grass;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockEvent>()
            .add_event::<RandomUpdateEvent>()
            .add_event::<SpawnFallingSandEvent>()
            .init_resource::<RandomTickSpeed>()
            .init_resource::<BlockUpdateEventQueue>()
            .add_systems(
                Update,
                (
                    (do_block_updates, set_block)
                        .chain()
                        .in_set(WorldSet),
                    add_mesh_to_falling_sand,
                    place_falling_sand
                        .after(PhysicsSystemSet::React)
                        .before(WorldSet),
                ),
            )
            .add_systems(FixedUpdate, (do_random_block_updates, spawn_falling_sand))
            .add_plugins((dirt::DirtUpdatePlugin, grass::GrassUpdatePlugin));
    }
}

#[derive(Event, Debug)]
pub struct SetBlockEvent {
    pub block: Block,
    pub world_pos: [i32; 3],
}

fn set_block(
    chunk_index: Res<ChunkIndex>,
    mut block_events: EventReader<SetBlockEvent>,
    mut q_blocks: Query<&mut Blocks>,
    mut block_updates: ResMut<BlockUpdateEventQueue>,
) {
    for event in block_events.read() {
        let [x, y, z] = event.world_pos;
        let chunk_size = CHUNK_SIZE_I32;

        let chunk_x = x.div_floor(chunk_size);
        let chunk_y = y.div_floor(chunk_size);
        let chunk_z = z.div_floor(chunk_size);

        let local_x = (x - chunk_x * chunk_size) as usize;
        let local_y = (y - chunk_y * chunk_size) as usize;
        let local_z = (z - chunk_z * chunk_size) as usize;
        let Some(entity) = chunk_index
            .entity_by_pos
            .get(&IVec3::new(chunk_x, chunk_y, chunk_z))
        else {
            continue;
        };
        let Some(mut blocks) = q_blocks.get_mut(*entity).ok() else {
            continue;
        };
        *blocks.at_pos_mut([local_x, local_y, local_z]) = event.block;
        block_updates.update_around(event.world_pos);
    }
}

/// The number of random updates per tick in a given chunk
#[derive(Resource)]
pub struct RandomTickSpeed(pub u8);

impl Default for RandomTickSpeed {
    fn default() -> Self {
        Self(24)
    }
}

#[derive(Event)]
pub struct RandomUpdateEvent {
    pub world_pos: IVec3,
    /// Position in the chunk
    pub local_pos: IVec3,
    pub chunk_id: Entity,
    pub block: Block,
}

#[derive(QueryData)]
struct RandomBlockUpdateQueryData {
    chunk_id: Entity,
    stage_neighborhood: &'static Neighborhood<Stage>,
    block_neighborhood: &'static Neighborhood<Blocks>,
    chunk_position: &'static ChunkPosition,
}

fn do_random_block_updates(
    q_chunk: Query<RandomBlockUpdateQueryData>,
    random_tick_speed: Res<RandomTickSpeed>,
    mut update_events: EventWriter<RandomUpdateEvent>,
) {
    let ticks = random_tick_speed.0;
    for chunk in q_chunk.iter() {
        if chunk
            .stage_neighborhood
            .min()
            .unwrap()
            .as_ref()
            != &Stage::final_stage()
        {
            continue;
        }
        for _ in 0..ticks {
            let local_pos = get_random_update_location();
            let Some(block) = chunk
                .block_neighborhood
                .at_pos(local_pos)
                .cloned()
            else {
                continue;
            };
            let chunk_id = chunk.chunk_id;
            let world_pos = local_pos + chunk.chunk_position.0 * CHUNK_SIZE_I32;
            let update = RandomUpdateEvent {
                world_pos,
                local_pos,
                chunk_id,
                block,
            };
            update_events.write(update);
        }
    }
}

fn get_random_update_location() -> IVec3 {
    [
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
    ]
    .map(|x| x % CHUNK_SIZE as u8)
    .map(|x| x as i32)
    .into()
}

/// Tells the game to update a block by checking its surroundings
struct BlockUpdateEvent {
    world_pos: IVec3,
}

impl<T: Into<IVec3>> From<T> for BlockUpdateEvent {
    fn from(value: T) -> Self {
        Self {
            world_pos: value.into(),
        }
    }
}

#[derive(Resource, Default)]
struct BlockUpdateEventQueue {
    queue: VecDeque<BlockUpdateEvent>,
}

impl BlockUpdateEventQueue {
    fn push(&mut self, event: impl Into<BlockUpdateEvent>) {
        self.queue.push_back(event.into());
    }

    fn pop(&mut self) -> Option<BlockUpdateEvent> {
        self.queue.pop_front()
    }

    fn update_around(&mut self, world_pos: impl Into<IVec3>) {
        let world_pos = world_pos.into();
        self.push(world_pos);
        self.push(world_pos + IVec3::X);
        self.push(world_pos - IVec3::X);
        self.push(world_pos + IVec3::Y);
        self.push(world_pos - IVec3::Y);
        self.push(world_pos + IVec3::Z);
        self.push(world_pos - IVec3::Z);
    }
}

fn do_block_updates(
    mut block_update_event_queue: ResMut<BlockUpdateEventQueue>,
    block_index: Res<ComponentIndex<Blocks>>,
    mut spawn_falling_sand_events: EventWriter<SpawnFallingSandEvent>,
) {
    while let Some(update) = block_update_event_queue.pop() {
        let world_pos = update.world_pos;
        let Some(block) = block_index.at_pos(world_pos) else {
            continue;
        };
        match block {
            Block::Sand if sand_should_fall(world_pos, block_index.as_ref()) => {
                spawn_falling_sand_events.write(world_pos.into());
            }
            _ => {}
        }
    }
}

fn sand_should_fall(world_pos: IVec3, block_index: &ComponentIndex<Blocks>) -> bool {
    let below = world_pos - IVec3::Y;
    match block_index.at_pos(below) {
        None | Some(Block::Air) => true,
        _ => false,
    }
}

#[derive(Event)]
struct SpawnFallingSandEvent {
    world_pos: IVec3,
}

impl<T: Into<IVec3>> From<T> for SpawnFallingSandEvent {
    fn from(value: T) -> Self {
        Self {
            world_pos: value.into(),
        }
    }
}

#[derive(Component)]
#[require(Gravity, Aabb::cube(0.9999), Collidable)]
struct FallingSand;

fn spawn_falling_sand(
    mut commands: Commands,
    mut sand_events: EventReader<SpawnFallingSandEvent>,
    mut set_block_events: EventWriter<SetBlockEvent>,
) {
    for SpawnFallingSandEvent { world_pos } in sand_events.read() {
        set_block_events.write(SetBlockEvent {
            block: Block::Air,
            world_pos: world_pos.to_array(),
        });
        let translation = world_pos.as_vec3() + Vec3::splat(0.5);
        commands.spawn((FallingSand, Transform::from_translation(translation)));
    }
}

#[derive(Component)]
struct Meshed;

fn add_mesh_to_falling_sand(
    mut commands: Commands,
    block_meshes: Res<BlockMeshes>,
    q_falling_sand: Query<Entity, (With<FallingSand>, Without<Meshed>)>,
) {
    let Some(mesh_data) = block_meshes.terrain.get(&Block::Sand) else {
        return;
    };
    let (mesh, material) = &mesh_data[0];
    for entity in q_falling_sand.iter() {
        commands
            .entity(entity)
            .insert((Visibility::Visible, Meshed))
            .with_child((
                mesh.clone(),
                material.clone(),
                Transform::from_translation(Vec3 {
                    x: -0.5,
                    y: 0.5,
                    z: -0.5,
                }),
            ));
    }
}

fn place_falling_sand(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    mut set_block_events: EventWriter<SetBlockEvent>,
    q_falling_sand: Query<(Entity, &Transform), With<FallingSand>>,
) {
    for event in collision_events.read() {
        let Ok((sand_id, sand_transform)) = q_falling_sand.get(event.entity) else {
            continue;
        };
        if event.normal.y <= 0.0 {
            // We only care if it's colliding with ground
            continue;
        }
        commands.entity(sand_id).despawn();
        let world_pos = sand_transform
            .translation
            .floor()
            .as_ivec3()
            .into();
        let set_block_event = SetBlockEvent {
            block: Block::Sand,
            world_pos,
        };
        set_block_events.write(set_block_event);
    }
}
