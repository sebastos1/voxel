use rand::Rng;
use bevy::prelude::*;
use crate::world::World;
use noise::{Perlin, NoiseFn};
use bevy::render::mesh::Indices;
use crate::shader::CustomMaterial;
use crate::voxel::{Voxel, BlockType};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

const CHUNK_SIZE: usize = 32;
const WORLD_HEIGHT: usize = 128;
const BASE_HEIGHT: f64 = 50.0;
const NOISE_SCALE: f64 = 0.05;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let mut world = World::new();

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let perlin = Perlin::new(seed);

    for chunk_x in 0..5 {
        for chunk_z in 0..5 {
            for chunk_y in 0..(WORLD_HEIGHT / CHUNK_SIZE) {
                let chunk_pos = (chunk_x, chunk_y as i32, chunk_z);
                let chunk = world.get_or_create_chunk(chunk_pos);
        
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let noise_value = perlin.get([
                            (x as f64 + chunk_pos.0 as f64 * CHUNK_SIZE as f64) * NOISE_SCALE, 
                            (z as f64 + chunk_pos.2 as f64 * CHUNK_SIZE as f64) * NOISE_SCALE, 
                            0.0 // Fixed to use a 2D slice of 3D noise
                        ]);
                        let noise_height = noise_value * 10.0;
                        let height = (BASE_HEIGHT + noise_height).round().max(0.0) as usize;
        
        
                        for y in 0..CHUNK_SIZE {
                            let world_y = chunk_y * CHUNK_SIZE + y;
                            let block_type = if world_y < height {
                                BlockType::Stone
                            } else if world_y == height {
                                BlockType::Dirt
                            } else {
                                BlockType::Air
                            };
                            chunk.voxels[x][y][z] = Voxel::new(block_type);
                        }
                    }
                }
        
                let colors = vec!(Color::RED, Color::BLUE, Color::GREEN, Color::YELLOW, Color::CYAN, Color::ORANGE);
                let chunk_meshes = chunk.generate_chunk_meshes();
                for (i, mesh) in chunk_meshes.iter().enumerate() {
                    commands.spawn(MaterialMeshBundle {
                        mesh: meshes.add(mesh.clone()),
                        material: materials.add(CustomMaterial { color: colors[i] }),
                        transform: Transform::from_translation(world_position(chunk_pos)),
                        ..Default::default()
                    });
                }
            }
        }
    }

    commands.insert_resource(world);
}

pub struct Chunk {
    pub voxels: [[[Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: [[[Voxel::new(BlockType::Air); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<&Voxel> {
        if x < 0 || x >= CHUNK_SIZE as i32 || y < 0 || y >= CHUNK_SIZE as i32 || z < 0 || z >= CHUNK_SIZE as i32 {
            return None;
        }
        Some(&self.voxels[x as usize][y as usize][z as usize])
    }

    pub fn is_face_visible(&self, x: i32, y: i32, z: i32, dx: i32, dy: i32, dz: i32) -> bool {
        let nx = x + dx;
        let ny = y + dy;
        let nz = z + dz;
        if nx < 0 || nx >= CHUNK_SIZE as i32 || ny < 0 || ny >= CHUNK_SIZE as i32 || nz < 0 || nz >= CHUNK_SIZE as i32 {
            return true; // Edge of the chunk, face is visible
        }
        self.voxels[nx as usize][ny as usize][nz as usize].is_transparent()
    }

    pub fn generate_chunk_meshes(&self) -> Vec<Mesh> {
        let mut dir_vertices = vec![Vec::new(); 6];
        let mut dir_indices = vec![Vec::new(); 6];

        let directions = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        let direction_colors: [[f32; 4]; 6] = [
            [1.0, 0.0, 0.0, 1.0], // Red for x+
            [1.0, 1.0, 0.0, 1.0], // Yellow for x-
            [0.0, 1.0, 0.0, 1.0], // Green for y+
            [0.0, 0.0, 1.0, 1.0], // Blue for y-
            [1.0, 0.5, 0.0, 1.0], // Orange for z+
            [0.5, 0.0, 0.5, 1.0], // Purple for z-
        ];

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let voxel = self.voxels[x][y][z];
                    if voxel.is_air() {
                        continue;
                    }

                    for (i, (dx, dy, dz)) in directions.iter().enumerate() {
                        if self.is_face_visible(x as i32, y as i32, z as i32, *dx, *dy, *dz) {
                            let base_index = dir_vertices[i].len() as u32; // Calculate current base index
                    
                            add_face_vertices(&mut dir_vertices[i], x, y, z, *dx, *dy, *dz, direction_colors[i]);
                    
                            dir_indices[i].extend_from_slice(&[
                                base_index, base_index + 1, base_index + 2,
                                base_index, base_index + 2, base_index + 3,
                            ]);
                        }
                    }
                }
            }
        }

        let mut meshes: Vec<Mesh> = Vec::new();
        for i in 0..6 {
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            let positions: Vec<[f32; 3]> = dir_vertices[i].iter().map(|v| v.0).collect();
            let colors: Vec<[f32; 4]> = dir_vertices[i].iter().map(|v| v.1).collect();

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_indices(Indices::U32(dir_indices[i].clone()));
            
            meshes.push(mesh);
        }
        meshes
    }
}

fn world_position(chunk_coords: (i32, i32, i32)) -> Vec3 {
    Vec3::new(
        chunk_coords.0 as f32 * CHUNK_SIZE as f32,
        chunk_coords.1 as f32 * CHUNK_SIZE as f32,
        chunk_coords.2 as f32 * CHUNK_SIZE as f32,
    )
}

fn add_face_vertices(vertices: &mut Vec<([f32; 3], [f32; 4])>, x: usize, y: usize, z: usize, dx: i32, dy: i32, dz: i32, color: [f32; 4]) {
    let base_x = x as f32;
    let base_y = y as f32;
    let base_z = z as f32;

    match (dx, dy, dz) {
        (1, 0, 0) => { // +X
            vertices.extend_from_slice(&[
                ([base_x + 1.0, base_y, base_z], color),
                ([base_x + 1.0, base_y + 1.0, base_z], color),
                ([base_x + 1.0, base_y + 1.0, base_z + 1.0], color),
                ([base_x + 1.0, base_y, base_z + 1.0], color),
            ]);
        },
        (-1, 0, 0) => { // -X
            vertices.extend_from_slice(&[
                ([base_x, base_y, base_z], color),
                ([base_x, base_y, base_z + 1.0], color),
                ([base_x, base_y + 1.0, base_z + 1.0], color),
                ([base_x, base_y + 1.0, base_z], color),
            ]);
        },
        (0, 1, 0) => { // +Y
            vertices.extend_from_slice(&[
                ([base_x, base_y + 1.0, base_z], color),
                ([base_x, base_y + 1.0, base_z + 1.0], color),
                ([base_x + 1.0, base_y + 1.0, base_z + 1.0], color),
                ([base_x + 1.0, base_y + 1.0, base_z], color),
            ]);
        },
        (0, -1, 0) => { // -Y
            vertices.extend_from_slice(&[
                ([base_x, base_y, base_z], color),
                ([base_x + 1.0, base_y, base_z], color),
                ([base_x + 1.0, base_y, base_z + 1.0], color),
                ([base_x, base_y, base_z + 1.0], color),
            ]);
        },
        (0, 0, 1) => { // +Z
            vertices.extend_from_slice(&[
                ([base_x, base_y, base_z + 1.0], color),
                ([base_x + 1.0, base_y, base_z + 1.0], color),
                ([base_x + 1.0, base_y + 1.0, base_z + 1.0], color),
                ([base_x, base_y + 1.0, base_z + 1.0], color),
            ]);
        },
        (0, 0, -1) => { // -Z
            vertices.extend_from_slice(&[
                ([base_x, base_y, base_z], color),
                ([base_x, base_y + 1.0, base_z], color),
                ([base_x + 1.0, base_y + 1.0, base_z], color),
                ([base_x + 1.0, base_y, base_z], color),
            ]);
        },
        _ => {}
    }
}