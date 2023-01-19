use vulkanalia::prelude::v1_0::*;

use super::appdata::AppData;
use super::queue_family_indices::QueueFamilyIndices;
use super::vertices::*;

use anyhow::{Result, Ok};

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> 
{

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
) -> Result<()> 
{
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    record_command_buffers(device, data)?;

    Ok(())
}

unsafe fn record_command_buffers(
    device: &Device, 
    data: &AppData
) -> Result<()> 
{
    for (i, command_buffer) in data.command_buffers.iter().enumerate() 
    {
        let inheritence = vk::CommandBufferInheritanceInfo::builder();
        let command_buffer_begin= vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty())
            .inheritance_info(&inheritence);

        device.begin_command_buffer(*command_buffer, &command_buffer_begin)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);
    
        let color_clear_value = vk::ClearValue 
        {
            color: vk::ClearColorValue 
            {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };
    
        let clear_values = &[color_clear_value];
        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &render_pass_begin, vk::SubpassContents::INLINE);

        device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);

        device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);

        device.cmd_draw(*command_buffer, VERTICES.len() as u32, 1, 0, 0);

        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}