use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

use super::appdata::AppData;



pub unsafe fn create_render_pass(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> 
{
    let colour_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(super::depth_objects::get_depth_format(instance, data)?)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let colour_resolve_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let colour_attachment_reference = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let depth_stencil_attachment_reference = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let colour_resolve_attachment_reference = vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let colour_attachments = &[colour_attachment_reference];
    let resolve_attachments = &[colour_resolve_attachment_reference];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(colour_attachments)
        .depth_stencil_attachment(&depth_stencil_attachment_reference)
        .resolve_attachments(resolve_attachments);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

    let attachments = &[
        colour_attachment, 
        depth_stencil_attachment,
        colour_resolve_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let create_info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments) 
        .subpasses(subpasses)
        .dependencies(dependencies);

    data.render_pass = device.create_render_pass(&create_info, None)?;
    
    Ok(())
}