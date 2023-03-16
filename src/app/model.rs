
use std::fs::File;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::BufReader;

use nalgebra_glm as glm;

use super::appdata::AppData;
use super::vertices::Vertex;

use anyhow::Result;

pub fn load_model(
    data: &mut AppData
) -> Result<()>
{
    let mut reader = BufReader::new(File::open("assets/models/viking_room.obj")?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader, 
        &tobj::LoadOptions { triangulate: true, ..Default::default() }, 
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

    for model in &models
    {
        for index in &model.mesh.indices
        {
            let position_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex
            {
                position: glm::vec3(
                    model.mesh.positions[position_offset], 
                    model.mesh.positions[position_offset + 1], 
                    model.mesh.positions[position_offset + 2]
                ),
                colour: glm::vec3(1.0, 1.0, 1.0),
                tex_coord: glm::vec2(
                    model.mesh.texcoords[tex_coord_offset], 
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1]
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex)
            {
                data.indices.push(*index as u32);
            }
            else
            {
                let index = data.vertices.len();
                unique_vertices.insert(vertex, index);
                data.vertices.push(vertex);
                data.indices.push(index as u32);
            }
        }
    }
    
    Ok(())
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool 
    {
        self.position == other.position
            && self.colour == other.colour
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.colour[0].to_bits().hash(state);
        self.colour[1].to_bits().hash(state);
        self.colour[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}
