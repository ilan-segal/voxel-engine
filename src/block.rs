use bevy::prelude::*;
use strum_macros::EnumIter;

pub const FLUID_DROP: f32 = -0.125;
pub const SURFACE_HEIGHT: f32 = 1.0 + FLUID_DROP;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, EnumIter)]
pub enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
    Sand,
    Wood,
    Leaves,
    Water,
    Bedrock,
}

// Required for Block to work as a key in hashmap operations `entry_ref` + `or_insert_with`
impl From<&Block> for Block {
    fn from(value: &Block) -> Self {
        *value
    }
}

impl Block {
    // pub fn get_colour(&self) -> Color {
    //     match self {
    //         Self::Grass => Color::linear_rgb(0.2, 0.6, 0.0),
    //         Self::Leaves => Color::hsv(124.0, 0.9, 0.39),
    //         Self::Water => Color::srgba(0.247, 0.463, 0.894, 0.5),
    //         _ => Color::WHITE,
    //     }
    // }

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

    pub fn is_translucent(&self) -> bool {
        match self {
            Self::Water => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Default)]
pub enum BlockSide {
    #[default]
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
