use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct SeedPlugin;

impl Plugin for SeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup.in_set(LoadSeed));
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadSeed;

#[derive(Resource, Serialize, Deserialize)]
pub struct WorldSeed(pub u32);

fn setup(mut commands: Commands) {
    commands.insert_resource(WorldSeed(0xDEADBEEF));
}
