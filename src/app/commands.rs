use vulkanalia::prelude::v1_0::*;

use super::{appdata::AppData, queue_family_indices::QueueFamilyIndices};

use anyhow::{Result, Ok};

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty())
        .queue_family_index(indices.graphics);

    data.command_pool = device.create_command_pool(&create_info, None)?;

    Ok(())
}

pub unsafe fn create_command_buffers(
    device: &Device, 
    data: &mut AppData
) -> Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;
    
    Ok(())
}