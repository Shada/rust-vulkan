use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use nalgebra_glm as glm;
use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;

use super::appdata::AppData;

use anyhow::{Result, anyhow};

use super::buffer::*;

lazy_static!
{
    pub static ref VERTICES: Vec<Vertex> = vec!
    [
        Vertex::new(glm::vec3(-0.5, -0.5, 0.0),  glm::vec3(1.0, 0.0, 0.0), glm::vec2(1.0, 0.0)),
        Vertex::new(glm::vec3(0.5,  -0.5, 0.0),  glm::vec3(0.0, 1.0, 0.0), glm::vec2(0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, 0.0),    glm::vec3(0.0, 0.0, 1.0), glm::vec2(0.0, 1.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, 0.0),   glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),

        Vertex::new(glm::vec3(-0.5, -0.5, -0.5),  glm::vec3(1.0, 0.0, 0.0), glm::vec2(1.0, 0.0)),
        Vertex::new(glm::vec3(0.5,  -0.5, -0.5),  glm::vec3(0.0, 1.0, 0.0), glm::vec2(0.0, 0.0)),
        Vertex::new(glm::vec3(0.5, 0.5, -0.5),    glm::vec3(0.0, 0.0, 1.0), glm::vec2(0.0, 1.0)),
        Vertex::new(glm::vec3(-0.5, 0.5, -0.5),   glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
    ];
}

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,
    4, 5, 6, 6, 7, 4
    ];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex
{
    position: glm::Vec3,
    colour: glm::Vec3,
    tex_coord: glm::Vec2,
}

impl Vertex
{
    fn new(position: glm::Vec3, colour: glm::Vec3, tex_coord: glm::Vec2) -> Self
    {
        Self
        {
            position,
            colour,
            tex_coord
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription
    {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }
    
    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3]
    {
        let position = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        let colour = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec3>() as u32)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<glm::Vec3>() + size_of::<glm::Vec3>()) as u32)
            .build();
        
        [position, colour, tex_coord]
    }

}

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    // Create Staging buffer

    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    // Copy data (staging)
    let memory = device.map_memory(
        staging_buffer_memory, 
        0, 
        size, 
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    device.unmap_memory(staging_buffer_memory);

    // Create Vertex buffer

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, 
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    // Copy (Vertex)
    
    copy_buffer(device, data, staging_buffer, vertex_buffer, size)?;

    // Cleanup

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}


pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    // Create Staging buffer

    let size = (size_of::<u16>() * INDICES.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, 
        vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    // Copy data (staging)
    let memory = device.map_memory(
        staging_buffer_memory, 
        0, 
        size, 
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(INDICES.as_ptr(), memory.cast(), INDICES.len());

    device.unmap_memory(staging_buffer_memory);

    // Create Index buffer

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, 
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    // Copy (Index)
    
    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    // Cleanup

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> 
{
    let memory = instance.get_physical_device_memory_properties(data.physical_device);

    (0..memory.memory_type_count)
        .find(|i| 
        {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find wuitable memory type."))
}