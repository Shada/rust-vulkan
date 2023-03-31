use std::{mem::size_of, time::Instant};

use nalgebra_glm as glm;

use vulkanalia::prelude::v1_0::*;

use anyhow::{Ok, Result};
use std::ptr::copy_nonoverlapping as memcpy;

use super::{appdata::AppData, buffer::create_buffer};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject
{
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

pub unsafe fn update_uniform_buffer(image_index: usize, start: &Instant, data: &AppData, device: &Device) -> Result<()>
{
    let time = start.elapsed().as_secs_f32();

    let view = glm::look_at(
        &glm::vec3(2.0, 2.0, 2.0), 
        &glm::vec3(0.0, 0.0, 0.0), 
        &glm::vec3(0.0, 0.0, 1.0),
    );

    let mut proj = glm::perspective_rh_zo(
        data.swapchain_extent.width as f32 / data.swapchain_extent.height as f32, 
        glm::radians(&glm::vec1(45.0))[0], 
        0.1, 
        10.0,
    );
    proj[(1, 1)] *= -1.0;

    let ubo = UniformBufferObject { view, proj };

    let memory = device.map_memory(
        data.uniform_buffers_memory[image_index], 
        0, 
        size_of::<UniformBufferObject>() as u64, 
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(&ubo, memory.cast(), 1);

    device.unmap_memory(data.uniform_buffers_memory[image_index]);

    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len()
    {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance, 
            device, 
            data, 
            size_of::<UniformBufferObject>() as u64, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }
    
    Ok(())
}

pub unsafe fn create_descriptor_set_layout(
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);
    
    let bindings = &[ubo_binding, sampler_binding];
    let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings);
    
    data.descriptor_set_layout = device.create_descriptor_set_layout(&create_info, None)?;

    Ok(())
}

pub unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()>
{
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32);
    
    let pool_sizes = &[ubo_size, sampler_size];
    let create_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32);

    data.descriptor_pool = device.create_descriptor_pool(&create_info, None)?;
    Ok(())
}

pub unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData)
    -> Result<()>
{
    let set_layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool) 
        .set_layouts(&set_layouts);
    
    data.descriptor_sets = device.allocate_descriptor_sets(&allocate_info)?;

    for i in 0..data.swapchain_images.len()
    {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);
            
        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_image_view)
            .sampler(data.texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.update_descriptor_sets(
            &[ubo_write, sampler_write], 
            &[] as &[vk::CopyDescriptorSet]
        );
    }
    Ok(())
}