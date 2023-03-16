use std::fs::File;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::{Result, Ok, anyhow};
use vulkanalia::prelude::v1_0::*;

use super::{appdata::AppData, buffer::create_buffer, vertices::get_memory_type_index, commands::{begin_single_time_commands, end_single_time_commands}};

pub unsafe fn create_texture_image(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result <()> 
{
    let image = File::open("assets/textures/texture.png")?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();
    
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance, 
        device, 
        data, 
        size, vk::BufferUsageFlags::TRANSFER_SRC, 
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(
        staging_buffer_memory, 
        0, 
        size, 
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

    device.unmap_memory(staging_buffer_memory);

    let (texture_image, texture_image_memory) = create_image(
        instance, 
        device, 
        data, 
        width, 
        height, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageTiling::OPTIMAL, 
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST, 
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.texture_image = texture_image;
    data.texture_image_memory = texture_image_memory;

    transition_image_layout(
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::UNDEFINED, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    )?;

    copy_buffer_to_image(
        device, 
        data, 
        staging_buffer, 
        data.texture_image, 
        width, 
        height,
    )?;

    transition_image_layout(
        device, 
        data, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok(())
}

pub unsafe fn create_texture_image_view(
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    data.texture_image_view = create_image_view(
        device, 
        data.texture_image, 
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
    )?;
    
    Ok(())
}

pub unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
) -> Result<vk::ImageView>
{
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let create_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&create_info, None)?)
}

pub unsafe fn create_texture_sampler(
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    let create_info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);

    data.texture_sampler = device.create_sampler(&create_info, None)?;
    
    Ok(())
}

pub unsafe fn transition_image_layout(
    device: &Device,
    data: &AppData,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<()>
{
    let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask,
    ) = match (old_layout, new_layout)
    {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        ),
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        _ => return Err(anyhow!("unsupprted image layout transition!")),
    };

    let command_buffer = begin_single_time_commands(device, data)?;

    let aspects_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL 
    {
        match format 
        {
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::DEPTH
        }
    }
    else
    {
        vk::ImageAspectFlags::COLOR
    };

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    device.cmd_pipeline_barrier(
        command_buffer, 
        src_stage_mask, 
        dst_stage_mask, 
        vk::DependencyFlags::empty(), 
        &[] as &[vk::MemoryBarrier], 
        &[] as &[vk::BufferMemoryBarrier], 
        &[barrier],
    );


    end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

unsafe fn copy_buffer_to_image(
    device: &Device,
    data: &AppData,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()>
{
    let command_buffer = begin_single_time_commands(device, data)?;

    let image_subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(image_subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0})
        .image_extent(vk::Extent3D { width, height, depth: 1});

    device.cmd_copy_buffer_to_image(
        command_buffer, 
        buffer, 
        image, 
        vk::ImageLayout::TRANSFER_DST_OPTIMAL, 
        &[region],
    );

    end_single_time_commands(device, data, command_buffer)?;
    Ok(())
}

pub unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    data: &AppData,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> 
{
    let create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = device.create_image(&create_info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            data,
            properties,
            requirements,
        )?);

    let image_memory = device.allocate_memory(&allocate_info, None)?;

    device.bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}
