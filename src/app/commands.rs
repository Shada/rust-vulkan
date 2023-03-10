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
        device.cmd_bind_index_buffer(*command_buffer, data.index_buffer, 0, vk::IndexType::UINT16);

        device.cmd_bind_descriptor_sets(
            *command_buffer, 
            vk::PipelineBindPoint::GRAPHICS, 
            data.pipeline_layout, 
            0, 
            &[data.descriptor_sets[i]], 
            &[],
        );

        device.cmd_draw_indexed(*command_buffer, INDICES.len() as u32, 1, 0, 0, 0);

        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}

pub unsafe fn begin_single_time_commands(
    device: &Device,
    data: &AppData,
) -> Result<vk::CommandBuffer>
{
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &begin_info)?;

    Ok(command_buffer)
}

pub unsafe fn end_single_time_commands(
    device: &Device,
    data: &AppData,
    command_buffer: vk::CommandBuffer,
) -> Result<()>
{
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(command_buffers);

    device.queue_submit(data.graphics_queue, &[submit_info], vk::Fence::null())?;
    device.queue_wait_idle(data.graphics_queue)?;

    device.free_command_buffers(data.command_pool, &[command_buffer]);

    Ok(())
}