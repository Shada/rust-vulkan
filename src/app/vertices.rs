use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use nalgebra_glm as glm;
use lazy_static::lazy_static;
use vulkanalia::prelude::v1_0::*;

use super::appdata::AppData;

use anyhow::{Result, anyhow};

lazy_static!
{
    pub static ref VERTICES: Vec<Vertex> = vec!
    [
        Vertex::new(glm::vec2(0.0, -0.5), glm::vec3(1.0, 0.0, 0.0)),
        Vertex::new(glm::vec2(0.5,  0.5), glm::vec3(0.0, 1.0, 0.0)),
        Vertex::new(glm::vec2(-0.5, 0.5), glm::vec3(1.0, 1.0, 1.0)),
    ];
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex
{
    position: glm::Vec2,
    colour: glm::Vec3,
}

impl Vertex
{
    fn new(position: glm::Vec2, colour: glm::Vec3) -> Self
    {
        Self
        {
            position,
            colour,
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
    
    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2]
    {
        let position = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let colour = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<glm::Vec2>() as u32)
            .build();
        
        [position, colour]
    }

}

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    // Buffer
    let buffer_info = vk::BufferCreateInfo::builder()
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    data.vertex_buffer = device.create_buffer(&buffer_info, None)?;

    // Memory 
    let requirements = device.get_buffer_memory_requirements(data.vertex_buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance, 
            data, 
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE, 
            requirements,
        )?);

    data.vertex_buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(data.vertex_buffer, data.vertex_buffer_memory, 0)?;

    // Copy data
    let memory = device.map_memory(
        data.vertex_buffer_memory, 
        0, 
        buffer_info.size, 
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());

    device.unmap_memory(data.vertex_buffer_memory);
    
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