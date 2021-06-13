mod debug;

use crate::debug::DebugPlugin;
use bevy::prelude::*;
use bevy::render::camera::PerspectiveProjection;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;

const CHUNK_H: usize = 8;
const CHUNK_V: usize = 16;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(block_update.system())
        .run();
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
struct Block {
    faces: [bool; 6],
    pos: [usize; 3],
}

#[derive(PartialEq, Copy, Clone)]
struct Chunk {
    blocks: [Option<Block>; CHUNK_H * CHUNK_H * CHUNK_V],
    pos: [isize; 2],
}

impl Chunk {
    pub fn new(x: isize, y: isize) -> Self {
        Chunk {
            blocks: [None; CHUNK_H * CHUNK_H * CHUNK_V],
            pos: [x, y],
        }
    }
    pub fn add(mut self, block: Block) -> Self {
        //TODO coords should be relative to chunk coords to also allow negative coord values for blocks.
        // implement chunk coords and set block coords relative within the chunk, so limited to CHUNK_H and CHUNK_V
        let pos = Self::coords_to_index(block.pos);
        if pos >= 0 && pos < (CHUNK_H * CHUNK_H * CHUNK_V) as isize {
            self.blocks[pos as usize] = Some(block);
        }
        self
    }
    pub fn coords_to_index(pos: [usize; 3]) -> isize {
        pos[0] as isize + (pos[1] * (CHUNK_H * CHUNK_H)) as isize + (pos[2] * CHUNK_H) as isize
    }
    pub fn neighbor_blocks(self, pos: [usize; 3]) -> [bool; 6] {
        let mut faces = [true, true, true, true, true, true];

        faces[0] = !self.has_block([pos[0], pos[1], pos[2].wrapping_add(1)]); // left
        faces[1] = !self.has_block([pos[0], pos[1], pos[2].wrapping_sub(1)]); // right
        faces[2] = !self.has_block([pos[0].wrapping_add(1), pos[1], pos[2]]); // down
        faces[3] = !self.has_block([pos[0].wrapping_sub(1), pos[1], pos[2]]); // up
        faces[4] = !self.has_block([pos[0], pos[1].wrapping_add(1), pos[2]]); // back
        faces[5] = !self.has_block([pos[0], pos[1].wrapping_sub(1), pos[2]]); // front
        faces
    }

    fn has_block(self, pos: [usize; 3]) -> bool {
        if pos[0] >= CHUNK_H
            || pos[1] >= CHUNK_V
            || pos[2] >= CHUNK_H
        {
            false
        } else {
            let index = Self::coords_to_index(pos);
            if index >= 0 && index <= (CHUNK_H * CHUNK_H * CHUNK_V) as isize {
                self.blocks[index as usize] != None
            } else {
                false
            }
        }
    }
}

fn setup(mut commands: Commands) {

    // spawn chunk
    let mut chunk = Chunk::new(0, 0);
    for x in 0..8 {
        for y in 0..16 {
            for z in 0..8 {
                chunk = chunk.add(Block {
                    pos: [x, y, z],
                    ..Default::default()
                });
            }
        }
    }
    commands.spawn().insert(chunk);

    // spawn chunk
    let mut chunk = Chunk::new(-1, 0);
    for x in 0..8 {
        for y in 0..16 {
            for z in 0..8 {
                if (x + y + z) % 2 == 0 {
                    chunk = chunk.add(Block {
                        pos: [x, y, z],
                        ..Default::default()
                    });
                }
            }
        }
    }
    commands.spawn().insert(chunk);

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(20., 25., -25.).looking_at(Vec3::new(0.,6.,0.), Vec3::Y),
        ..Default::default()
    });
}

fn block_update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<&mut Chunk, Changed<Chunk>>,
) {
    query.for_each_mut(|chunk| {
        for block in chunk.blocks.iter().flatten() {
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(gen_cube(chunk.neighbor_blocks(block.pos))),
                material: materials.add(StandardMaterial {
                    // base_color_texture: materials.add(block1.texture),
                    ..Default::default()
                }),
                transform: Transform::from_xyz(
                    (block.pos[0] as isize + (chunk.pos[0] * (CHUNK_H as isize))) as f32,
                    block.pos[1] as f32,
                    (block.pos[2] as isize + (chunk.pos[1] * (CHUNK_H as isize))) as f32,
                ),
                ..Default::default()
            });
        }
    })
}

fn gen_cube(faces: [bool; 6]) -> Mesh {
    let sp = shape::Box::new(0.9, 0.9, 0.9);

    let vertices = &[
        // Top
        ([sp.min_x, sp.min_y, sp.max_z], [0., 0., 1.0], [0., 0.]),
        ([sp.max_x, sp.min_y, sp.max_z], [0., 0., 1.0], [1.0, 0.]),
        ([sp.max_x, sp.max_y, sp.max_z], [0., 0., 1.0], [1.0, 1.0]),
        ([sp.min_x, sp.max_y, sp.max_z], [0., 0., 1.0], [0., 1.0]),
        // Bottom
        ([sp.min_x, sp.max_y, sp.min_z], [0., 0., -1.0], [1.0, 0.]),
        ([sp.max_x, sp.max_y, sp.min_z], [0., 0., -1.0], [0., 0.]),
        ([sp.max_x, sp.min_y, sp.min_z], [0., 0., -1.0], [0., 1.0]),
        ([sp.min_x, sp.min_y, sp.min_z], [0., 0., -1.0], [1.0, 1.0]),
        // Right
        ([sp.max_x, sp.min_y, sp.min_z], [1.0, 0., 0.], [0., 0.]),
        ([sp.max_x, sp.max_y, sp.min_z], [1.0, 0., 0.], [1.0, 0.]),
        ([sp.max_x, sp.max_y, sp.max_z], [1.0, 0., 0.], [1.0, 1.0]),
        ([sp.max_x, sp.min_y, sp.max_z], [1.0, 0., 0.], [0., 1.0]),
        // Left
        ([sp.min_x, sp.min_y, sp.max_z], [-1.0, 0., 0.], [1.0, 0.]),
        ([sp.min_x, sp.max_y, sp.max_z], [-1.0, 0., 0.], [0., 0.]),
        ([sp.min_x, sp.max_y, sp.min_z], [-1.0, 0., 0.], [0., 1.0]),
        ([sp.min_x, sp.min_y, sp.min_z], [-1.0, 0., 0.], [1.0, 1.0]),
        // Front
        ([sp.max_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [1.0, 0.]),
        ([sp.min_x, sp.max_y, sp.min_z], [0., 1.0, 0.], [0., 0.]),
        ([sp.min_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [0., 1.0]),
        ([sp.max_x, sp.max_y, sp.max_z], [0., 1.0, 0.], [1.0, 1.0]),
        // Back
        ([sp.max_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [0., 0.]),
        ([sp.min_x, sp.min_y, sp.max_z], [0., -1.0, 0.], [1.0, 0.]),
        ([sp.min_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [1.0, 1.0]),
        ([sp.max_x, sp.min_y, sp.min_z], [0., -1.0, 0.], [0., 1.0]),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);

    for (pos, (position, normal, uv)) in vertices.iter().enumerate() {
        if faces[pos / 4] {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }
    }

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ]);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));
    mesh
}
