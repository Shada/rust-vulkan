
use vulkanalia::prelude::v1_0::*;

use super::{appdata::AppData, vertices::get_memory_type_index, commands::{begin_single_time_commands, end_single_time_commands}};

use anyhow::{Result, Ok};

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> 
{
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    
        let buffer = device.create_buffer(&buffer_info, None)?;

        let requirements = device.get_buffer_memory_requirements(buffer);

        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(get_memory_type_index(
                instance,
                data, 
                properties, 
                requirements
            )?);

        let buffer_memory = device.allocate_memory(&memory_info, None)?;

        device.bind_buffer_memory(buffer, buffer_memory, 0)?;

        Ok((buffer, buffer_memory))
}

pub unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()>
{
    let command_buffer = begin_single_time_commands(device, data)?;

    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}