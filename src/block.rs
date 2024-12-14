use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::chunk::{index::ChunkIndex, ChunkUpdate, CHUNK_SIZE};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetBlockEvent>()
            .add_systems(Update, set_block);
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Deserialize, Serialize, Hash)]
pub enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
}

// Required for Block to work as a key in hashmap operations `entry_ref` + `or_insert_with`
impl From<&Block> for Block {
    fn from(value: &Block) -> Self {
        *value
    }
}

impl Block {
    pub fn get_colour(&self) -> Option<Color> {
        match self {
            Self::Stone => Some(Color::linear_rgb(0.2, 0.2, 0.2)),
            Self::Grass => Some(Color::linear_rgb(0.2, 0.6, 0.0)),
            Self::Dirt => Some(Color::hsv(35.0, 0.65, 0.65)),
            _ => None,
        }
    }

    pub fn is_meshable(&self) -> bool {
        match self {
            Self::Air => false,
            _ => true,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            Self::Air => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub enum BlockSide {
    Up,
    Down,
    North,
    South,
    West,
    East,
}

impl From<Dir3> for BlockSide {
    fn from(value: Dir3) -> Self {
        let closest = [
            Dir3::X,
            Dir3::NEG_X,
            Dir3::Y,
            Dir3::NEG_Y,
            Dir3::Z,
            Dir3::NEG_Z,
        ]
        .iter()
        .min_by(|ax1, ax2| {
            let d1 = (ax1.as_vec3() - value.as_vec3()).length();
            let d2 = (ax2.as_vec3() - value.as_vec3()).length();
            d1.total_cmp(&d2)
        })
        .unwrap();
        match closest {
            &Dir3::X => Self::North,
            &Dir3::NEG_X => Self::South,
            &Dir3::Y => Self::Up,
            &Dir3::NEG_Y => Self::Down,
            &Dir3::Z => Self::East,
            &Dir3::NEG_Z => Self::West,
            _ => panic!("Unexpected non-axis direction {:?}", closest),
        }
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
    mut chunk_events: EventWriter<ChunkUpdate>,
) {
    for event in block_events.read() {
        let [x, y, z] = event.world_pos;
        let chunk_size = CHUNK_SIZE as i32;
        let chunk_x = x.div_floor(chunk_size);
        let chunk_y = y.div_floor(chunk_size);
        let chunk_z = z.div_floor(chunk_size);
        let Some(chunk) = chunk_index.get_chunk(chunk_x, chunk_y, chunk_z) else {
            warn!(
                "Unable to process event due to non-existent chunk {:?}",
                event
            );
            continue;
        };
        info!("Deleting block at {:?}", event.world_pos);
        let local_x = (x - chunk_x * chunk_size) as usize;
        let local_y = (y - chunk_y * chunk_size) as usize;
        let local_z = (z - chunk_z * chunk_size) as usize;
        chunk
            .write()
            .expect("Write lock on chunk data")
            .put(local_x, local_y, local_z, event.block);
        chunk_events.send(ChunkUpdate::new(chunk_x, chunk_y, chunk_z));
    }
}
