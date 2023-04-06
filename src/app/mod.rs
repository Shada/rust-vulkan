
use anyhow::{anyhow, Result};

use std::collections::HashSet;
use std::time::Instant;

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::prelude::v1_0::*;

use winit::window::Window;

mod appdata;
mod buffer;
mod colour_objects;
mod commands;
mod debug_callback;
mod depth_objects;
mod framebuffers;
mod model;
mod physical_device;
mod pipeline;
mod renderpass;
mod suitability_error;
mod swapchain;
mod sync_objects;
mod texture;
mod uniform_buffer;
mod vertices;
mod queue_family_indices;

const MAX_FRAMES_IN_FLIGHT: usize = 2;
const VALIDATION_ENABLED: bool = true;
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App 
{
    entry: Entry,
    instance: Instance,
    data: appdata::AppData,
    device: Device,
    frame: usize,
    pub resized: bool,
    start: Instant, 
}

impl App 
{
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> Result<Self> 
    {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = appdata::AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, window)?;

        physical_device::pick_physical_device(&instance, &mut data)?;

        let device = create_logical_device(&instance, &mut data)?;

        swapchain::create_swapchain(window, &instance, &device, &mut data)?;
        swapchain::create_swapchain_image_views(&device, &mut data)?;

        renderpass::create_render_pass(&instance, &device, &mut data)?;
        uniform_buffer::create_descriptor_set_layout(&device, &mut data)?;
        pipeline::create_pipeline(&device, &mut data)?;

        commands::create_command_pool(&instance, &device, &mut data)?;
        
        colour_objects::create_colour_objects(&instance, &device, &mut data)?;
        depth_objects::create_depth_objects(&instance, &device, &mut data)?;
        framebuffers::create_framebuffers(&device, &mut data)?;

        texture::create_texture_image(&instance, &device, &mut data)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        model::load_model(&mut data)?;
        
        vertices::create_vertex_buffer(&instance, &device, &mut data)?;
        vertices::create_index_buffer(&instance, &device, &mut data)?;
        uniform_buffer::create_uniform_buffers(&instance, &device, &mut data)?;
        uniform_buffer::create_descriptor_pool(&device, &mut data)?;
        uniform_buffer::create_descriptor_sets(&device, &mut data)?;

        commands::create_command_buffers(&device, &mut data)?;

        sync_objects::create_sync_objects(&device, &mut data)?;

        Ok(Self 
        { 
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> Result<()> 
    {
        let in_flight_fence = self.data.in_flight_fences[self.frame];

        self.device
            .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;

        // Aquire swapchain image
        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::max_value(),
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() 
        {
            self.device
                .wait_for_fences(
                    &[image_in_flight], 
                    true, 
                    u64::max_value())?;
        }

        self.data.images_in_flight[image_index as usize] = in_flight_fence;
        
        let time = self.start.elapsed().as_secs_f32();

        commands::update_command_buffer(&mut self.data, &self.device, image_index, time)?;
        uniform_buffer::update_uniform_buffer(image_index, &self.start, &self.data, &self.device)?;

        //Submit command buffer
        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.reset_fences(&[in_flight_fence])?;

        self.device
            .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?;
            
        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT; 

        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) 
    {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.device.destroy_sampler(self.data.texture_sampler, None);
        self.device.destroy_image_view(self.data.texture_image_view, None);
        self.device.destroy_image(self.data.texture_image, None);
        self.device.free_memory(self.data.texture_image_memory, None);

        self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);

        self.device.destroy_buffer(self.data.index_buffer, None);
        self.device.free_memory(self.data.index_buffer_memory, None);
        self.device.destroy_buffer(self.data.vertex_buffer, None);
        self.device.free_memory(self.data.vertex_buffer_memory, None);

        self.data.in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.destroy_command_pool(self.data.command_pool, None);
        
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);

        if VALIDATION_ENABLED 
        {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()>
    {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        swapchain::create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        swapchain::create_swapchain_image_views(&self.device, &mut self.data)?;
        renderpass::create_render_pass(&self.instance, &self.device, &mut self.data)?;
        pipeline::create_pipeline(&self.device, &mut self.data)?;
        colour_objects::create_colour_objects(&self.instance, &self.device, &mut self.data)?;
        depth_objects::create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        framebuffers::create_framebuffers(&self.device, &mut self.data)?;
        uniform_buffer::create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        uniform_buffer::create_descriptor_pool(&self.device, &mut self.data)?;
        uniform_buffer::create_descriptor_sets(&self.device, &mut self.data)?;
        commands::create_command_buffers(&self.device, &mut self.data)?;
        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self)
    {
        self.device.destroy_image_view(self.data.colour_image_view, None);
        self.device.free_memory(self.data.colour_image_memory, None);
        self.device.destroy_image(self.data.colour_image, None);

        self.device.destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);

        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data.uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data.uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.data.framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data.swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}

/// Creates Vulkan instance
unsafe fn create_instance(
    window: &Window, 
    entry: &Entry,
    data: &mut appdata::AppData
) -> Result<Instance> 
{
    // Optional Application information
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    // Validation layers extensions
    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) 
    {
        return Err(anyhow!("Validationlayer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED 
    {
        vec![VALIDATION_LAYER.as_ptr()]
    }
    else 
    {
        Vec::new()
    };

    // Enumerate required global extensions
    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED 
    {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Define Create info using the application information and global extensions
    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);
    
    // Create debug info
    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | 
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL | 
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
        )
        .user_callback(Some(debug_callback::debug_callback));

    if VALIDATION_ENABLED 
    {
        info = info.push_next(&mut debug_info);
    }
    
    // Create instance
    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED 
    {
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

// create logical device
unsafe fn create_logical_device(
    instance: &Instance,
    data: &mut appdata::AppData,
) -> Result<Device> 
{
    let indices = queue_family_indices::QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| 
        {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if VALIDATION_ENABLED 
    {
        vec![VALIDATION_LAYER.as_ptr()]
    } else 
    {
        vec![]
    };

    let extensions = physical_device::DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        // Enable sample shading feature for the device.
        .sample_rate_shading(true);

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(data.physical_device, &info, None)?;

    data.graphics_queue = device.get_device_queue(indices.graphics, 0);
    data.present_queue = device.get_device_queue(indices.present, 0);

    Ok(device)
}
