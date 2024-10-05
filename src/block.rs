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
