use vulkanalia::prelude::v1_0::*;

use nalgebra_glm as glm;

use super::appdata::AppData;
use super::queue_family_indices::QueueFamilyIndices;

use anyhow::{Result, Ok};

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> 
{

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
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

    Ok(())
}

pub unsafe fn update_command_buffer(
    data: &mut AppData,
    device: &Device,
    image_index: usize,
    time: f32,
) -> Result<()>
{
    let command_buffer = data.command_buffers[image_index];

    device.reset_command_buffer(
        command_buffer, 
        vk::CommandBufferResetFlags::empty()
    )?;

    record_command_buffer(device, data, image_index, time)?;

    Ok(())
}

unsafe fn record_command_buffer(
    device: &Device, 
    data: &AppData,
    image_index: usize,
    time: f32,
) -> Result<()> 
{
    let command_buffer = data.command_buffers[image_index];

    let model = glm::rotate(
        &glm::identity(),
        time * glm::radians(&glm::vec1(90.0))[0],
        &glm::vec3(0.0, 0.0, 1.0));
    let (_, model_bytes, _) = model.as_slice().align_to::<u8>();

    let info = vk::CommandBufferBeginInfo::builder();

    device.begin_command_buffer(command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(data.swapchain_extent);

    let color_clear_value = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let depth_clear_value = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
    };

    let clear_values = &[color_clear_value, depth_clear_value];
    let info = vk::RenderPassBeginInfo::builder()
        .render_pass(data.render_pass)
        .framebuffer(data.framebuffers[image_index])
        .render_area(render_area)
        .clear_values(clear_values);

    device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
    device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, data.pipeline);
    device.cmd_bind_vertex_buffers(command_buffer, 0, &[data.vertex_buffer], &[0]);
    device.cmd_bind_index_buffer(command_buffer, data.index_buffer, 0, vk::IndexType::UINT32);
    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        data.pipeline_layout,
        0,
        &[data.descriptor_sets[image_index]],
        &[],
    );
    device.cmd_push_constants(
        command_buffer,
        data.pipeline_layout,
        vk::ShaderStageFlags::VERTEX,
        0,
        model_bytes,
    );
    device.cmd_push_constants(
        command_buffer,
        data.pipeline_layout,
        vk::ShaderStageFlags::FRAGMENT,
        64,
        &0.25f32.to_ne_bytes()[..],
    );
    device.cmd_draw_indexed(command_buffer, data.indices.len() as u32, 1, 0, 0, 0);
    device.cmd_end_render_pass(command_buffer);

    device.end_command_buffer(command_buffer)?;

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