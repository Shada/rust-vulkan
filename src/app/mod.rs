mod debug_callback;

mod vertices;
mod buffer;

mod uniform_buffer;

mod texture;
use texture::*;

mod physical_device;
use physical_device::*;

mod swapchain;
use self::commands::create_command_buffers;
use self::swapchain::{create_swapchain, create_swapchain_image_views};
use self::vertices::{create_vertex_buffer, create_index_buffer};

mod sync_objects;
use sync_objects::create_sync_objects;

mod commands;
use commands::create_command_pool;

mod renderpass;
use renderpass::create_render_pass;

mod framebuffers;
use framebuffers::create_framebuffers;

mod pipeline;
use pipeline::create_pipeline;

mod appdata;
use appdata::*;

mod queue_family_indices;
use queue_family_indices::*;

mod suitability_error;

use nalgebra_glm as glm;

use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::{anyhow, Result};

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;

use winit::window::Window;

use std::collections::HashSet;
use std::time::Instant;

use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension};

use self::debug_callback::debug_callback;


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
    data: AppData,
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
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, window)?;

        pick_physical_device(&instance, &mut data)?;

        let device = create_logical_device(&instance, &mut data)?;

        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;

        create_render_pass(&instance, &device, &mut data)?;
        uniform_buffer::create_descriptor_set_layout(&device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        create_framebuffers(&device, &mut data)?;

        create_command_pool(&instance, &device, &mut data)?;
        create_texture_image(&instance, &device, &mut data)?;

        create_vertex_buffer(&instance, &device, &mut data)?;
        create_index_buffer(&instance, &device, &mut data)?;
        uniform_buffer::create_uniform_buffers(&instance, &device, &mut data)?;
        uniform_buffer::create_descriptor_pool(&device, &mut data)?;
        uniform_buffer::create_descriptor_sets(&device, &mut data)?;

        create_command_buffers(&device, &mut data)?;

        create_sync_objects(&device, &mut data)?;

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
        
        self.update_uniform_buffer(image_index)?;

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

    // TODO: move to uniform_buffers.rs
    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()>
    {
        let time = self.start.elapsed().as_secs_f32();

        let model = glm::rotate(
            &glm::identity(),
            time * glm::radians(&glm::vec1(90.0))[0],
            &glm::vec3(0.0,0.0,1.0),
        );

        let view = glm::look_at(
            &glm::vec3(2.0, 2.0, 2.0), 
            &glm::vec3(0.0, 0.0, 0.0), 
            &glm::vec3(0.0, 0.0, 1.0),
        );

        let mut proj = glm::perspective(
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32, 
            glm::radians(&glm::vec1(45.0))[0], 
            0.1, 
            10.0,
        );
        proj[(1, 1)] *= -1.0;

        let ubo = uniform_buffer::UniformBufferObject { model, view, proj };

        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index], 
            0, 
            size_of::<uniform_buffer::UniformBufferObject>() as u64, 
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(self.data.uniform_buffers_memory[image_index]);
        Ok(())
    }

    /// Destroys our Vulkan app.
    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) 
    {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

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
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        uniform_buffer::create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        uniform_buffer::create_descriptor_pool(&self.device, &mut self.data)?;
        uniform_buffer::create_descriptor_sets(&self.device, &mut self.data)?;
        create_command_buffers(&self.device, &mut self.data)?;
        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self)
    {
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
    data: &mut AppData
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
        .user_callback(Some(debug_callback));

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
    data: &mut AppData,
) -> Result<Device> 
{
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

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

    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    let features = vk::PhysicalDeviceFeatures::builder();

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