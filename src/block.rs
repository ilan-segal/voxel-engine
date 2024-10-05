use bevy::{
    color::palettes::{
        basic::{GREEN, SILVER},
        css::BROWN,
    },
    prelude::*,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    #[default]
    Air,
    Stone,
    Dirt,
    Grass,
}

impl Block {
    pub fn get_colour(&self) -> Option<Color> {
        match self {
            Self::Stone => Some(SILVER),
            Self::Grass => Some(GREEN),
            Self::Dirt => Some(BROWN),
            _ => None,
        }
        .map(Color::from)
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
