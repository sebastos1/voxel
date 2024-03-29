use crate::chunk::Chunk;
use bevy::utils::HashMap;
use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<(i32, i32, i32), Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn get_or_create_chunk(&mut self, chunk_pos: (i32, i32, i32)) -> &mut Chunk {
        self.chunks.entry(chunk_pos).or_insert_with(Chunk::new)
    }
}