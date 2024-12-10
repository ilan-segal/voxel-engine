use std::fs::File;

use bevy::{
    ecs::query::QueryData,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use chunk_file_data::ChunkFileData;

use crate::chunk::{position::ChunkPosition, Chunk};

mod block_spec;
mod chunk_file_data;

pub struct FileIOPlugin;

impl Plugin for FileIOPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkSaveTasks>()
            .observe(save_chunk_on_unload);
    }
}

#[derive(Resource, Default)]
struct ChunkSaveTasks(HashMap<ChunkPosition, Task<()>>);

#[derive(QueryData)]
struct ChunkUnloadQuery {
    pos: &'static ChunkPosition,
    chunk: &'static Chunk,
}

fn save_chunk_on_unload(
    trigger: Trigger<OnRemove, Chunk>,
    q: Query<ChunkUnloadQuery>,
    mut tasks: ResMut<ChunkSaveTasks>,
) {
    let entity = trigger.entity();
    let Ok(data) = q.get(entity) else {
        error!(
            "Failed to save chunk data for entity {:?}; entity no longer exists!",
            entity
        );
        return;
    };
    if let Some(..) = tasks.0.remove(data.pos) {
        warn!(
            "Interrupting save task with another save task for chunk at position {:?}",
            data.pos.0
        );
    }
    let task_pool = AsyncComputeTaskPool::get();
    let cloned_pos = data.pos.clone();
    let cloned_chunk = data.chunk.clone();
    let task = task_pool.spawn(async move {
        match save_chunk_to_file(cloned_pos, cloned_chunk) {
            Err(message) => error!("Failed to save chunk to file; {:}", message),
            Ok(..) => info!("Saved chunk at {:?}", cloned_pos.0),
        }
    });
    tasks.0.insert(*data.pos, task);
}

fn save_chunk_to_file(pos: ChunkPosition, chunk: Chunk) -> Result<(), String> {
    const WORLD_FOLDER: &str = "world"; // TODO: Dynamically select world folder
    const WORLD_DIMENSION_NAME: &str = "overworld";
    let filename = format!(
        "{:}/{:}_{:}_{:}_{:}.chunkdata",
        WORLD_FOLDER, WORLD_DIMENSION_NAME, pos.0.x, pos.0.y, pos.0.z
    );
    let file = File::create(&filename).map_err(|err| format!("{:?}", err))?;
    let file_data: ChunkFileData = chunk.blocks.as_ref().into();
    serde_json::to_writer(file, &file_data).map_err(|err| format!("{:?}", err))?;
    Ok(())
}
