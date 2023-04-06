use vulkanalia::prelude::v1_0::*;

use nalgebra_glm as glm;

use super::appdata::AppData;
use super::queue_family_indices::QueueFamilyIndices;

use anyhow::{Result, Ok};

// Commmand pools for short-lived command buffers
unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<vk::CommandPool> 
{
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&create_info, None)?)
}

pub unsafe fn create_command_pools(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> 
{
    // Global
    data.command_pool = create_command_pool(instance, device, data)?;

    // Per-framebuffer
    let num_images = data.swapchain_images.len();
    for _ in 0..num_images
    {
        let command_pool = create_command_pool(instance, device, data)?;
        data.command_pools.push(command_pool);
    }

    Ok(())
}

pub unsafe fn create_command_buffers(
    device: &Device, 
    data: &mut AppData
) -> Result<()> 
{
    let num_images = data.swapchain_images.len();
    for image_index in 0..num_images
    {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(data.command_pools[image_index])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];    
        data.command_buffers.push(command_buffer);
    }
    
    Ok(())
}

pub unsafe fn update_command_buffer(
    app: &mut super::App,
    image_index: usize,
    time: f32,
) -> Result<()>
{
    let command_pool = app.data.command_pools[image_index];
    app.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

    let command_buffer = app.data.command_buffers[image_index];
    
    record_command_buffer(app, image_index, time)?;

    Ok(())
}

//unsafe fn update_secondary_command_buffer(
//    app: &mut super::App,
//    image_index: usize,
//    model_index: usize,
//) -> Result<vk::CommandBuffer>
//{
//    app.data.secondary_command_buffers.resize_with(image_index + 1, Vec::new);
//
//    let command_buffers = &mut app.data.secondary_command_buffers[image_index];
//
//    //while model_index >= command_buffers.len()
//    //{
//    //    let allocate_info = vk::CommandBufferAllocateInfo::builder()
//    //        .command_pool(app.data.command_pool)
//    //}
//
//    Ok(command_buffers[0])
//}

unsafe fn record_command_buffer(
    app: &mut super::App,
    image_index: usize,
    time: f32,
) -> Result<()> 
{
    let command_buffer = app.data.command_buffers[image_index];

    let model = glm::rotate(
        &glm::identity(),
        time * glm::radians(&glm::vec1(90.0))[0],
        &glm::vec3(0.0, 0.0, 1.0));
    let (_, model_bytes, _) = model.as_slice().align_to::<u8>();

    let info = vk::CommandBufferBeginInfo::builder();

    app.device.begin_command_buffer(command_buffer, &info)?;

    let render_area = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(app.data.swapchain_extent);

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
        .render_pass(app.data.render_pass)
        .framebuffer(app.data.framebuffers[image_index])
        .render_area(render_area)
        .clear_values(clear_values);

    app.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
    app.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, app.data.pipeline);
    app.device.cmd_bind_vertex_buffers(command_buffer, 0, &[app.data.vertex_buffer], &[0]);
    app.device.cmd_bind_index_buffer(command_buffer, app.data.index_buffer, 0, vk::IndexType::UINT32);
    app.device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        app.data.pipeline_layout,
        0,
        &[app.data.descriptor_sets[image_index]],
        &[],
    );
    app.device.cmd_push_constants(
        command_buffer,
        app.data.pipeline_layout,
        vk::ShaderStageFlags::VERTEX,
        0,
        model_bytes,
    );
    app.device.cmd_push_constants(
        command_buffer,
        app.data.pipeline_layout,
        vk::ShaderStageFlags::FRAGMENT,
        64,
        &0.25f32.to_ne_bytes()[..],
    );
    app.device.cmd_draw_indexed(command_buffer, app.data.indices.len() as u32, 1, 0, 0, 0);
    app.device.cmd_end_render_pass(command_buffer);

    app.device.end_command_buffer(command_buffer)?;

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