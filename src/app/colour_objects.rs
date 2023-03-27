use vulkanalia::prelude::v1_0::*;

use super::appdata::AppData;
use super::texture::{create_image, create_image_view};

use anyhow::Result;

pub unsafe fn create_colour_objects(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()>
{
    let (colour_image, colour_image_memory) = create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        data.msaa_samples,
        data.swapchain_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.colour_image = colour_image;
    data.colour_image_memory = colour_image_memory;

    data.colour_image_view = create_image_view(
        device,
        data.colour_image,
        data.swapchain_format,
        vk::ImageAspectFlags::COLOR,
        1,
    )?;

    Ok(())
}