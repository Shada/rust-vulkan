use super::appdata::AppData;
use super::queue_family_indices::QueueFamilyIndices;
use super::swapchain::SwapChainSupport;
use super::suitability_error::SuitabilityError;

use anyhow::{anyhow, Result, Ok};

use vulkanalia::prelude::v1_0::*;

use std::collections::HashSet;

// TODO: check the properties and select the best Device. 
// TODO: implement configuration options, where the available options is dynamically updated based on selected device

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> 
{
    for physical_device in instance.enumerate_physical_devices()? 
    {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) 
        {
            warn!("Skipping physical device (`{}`): {}", properties.device_name, error);
        } else 
        {
            info!("Selected physical device (`{}`).", properties.device_name);

            data.physical_device = physical_device;
            data.msaa_samples = get_max_msaa_samples(instance, data);

            return Ok(())
        }
    }
    
    Err(anyhow!("Failed to find suitable physical device."))
}

pub unsafe fn get_max_msaa_samples(
    instance: &Instance,
    data: &AppData,
) -> vk::SampleCountFlags
{
    let properties = instance.get_physical_device_properties(data.physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
        .iter()
        .cloned()
        .find(|c| counts.contains(*c))
        .unwrap_or(vk::SampleCountFlags::_1)
}

// TODO: change to ranking
pub unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> 
{
    let properties = instance
        .get_physical_device_properties(physical_device);
    //if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU 
    //{
    //    return Err(anyhow!(SuitabilityError("Only discrete GPUs are supported.")));
    //}

    let features = instance
        .get_physical_device_features(physical_device);
    //if features.geometry_shader != vk::TRUE 
    //{
    //    return Err(anyhow!(SuitabilityError("Missing geometry shader support.")));
    //}

    if features.sampler_anisotropy != vk::TRUE
    {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy!")));
    }

    QueueFamilyIndices::get(instance, data, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapChainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() 
    {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = instance
        .get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE
    {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy!")));
    }
    
    Ok(())
}

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

pub unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> 
{
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) 
    {
        Ok(())
    } else 
    {
        Err(anyhow!(SuitabilityError("Missing required device extensions.")))
    }
}