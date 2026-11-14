//! Retained WebGPU buffers for the prism draw graph.
//! WGSL reflection defines uniforms; browser WebIDL defines resource flags.
use crate::packing::{PackingError, try_pack};
use crate::reflection::HostShareableLayout;
use js::{ArrayBuffer, Float32Array, Unknown};
use react::webapi::{GPUBuffer, GPUBufferDescriptor, GPUDevice, GPUQueue, gpu_buffer_usage};

pub struct UniformBuffer {
    pub buffer: &'static GPUBuffer,
    pub layout: &'static HostShareableLayout,
}

/// Validate before allocating a GPU resource, then upload the complete block.
pub fn create_uniform_buffer(
    device: &GPUDevice,
    layout: &'static HostShareableLayout,
    value: Option<&Unknown>,
    label: &str,
) -> Result<UniformBuffer, PackingError> {
    let bytes = try_pack(layout, value)?;
    Ok(create_packed_uniform_buffer(device, layout, bytes, label))
}

pub(crate) fn create_packed_uniform_buffer(
    device: &GPUDevice,
    layout: &'static HostShareableLayout,
    bytes: &ArrayBuffer,
    label: &str,
) -> UniformBuffer {
    let buffer = device.create_buffer(GPUBufferDescriptor {
        label: Some(label),
        size: bytes.byte_length(),
        usage: f64::from(gpu_buffer_usage::UNIFORM | gpu_buffer_usage::COPY_DST),
        mapped_at_creation: None,
    });
    device.queue().write_buffer(buffer, 0, bytes, None, None);
    UniformBuffer { buffer, layout }
}

/// Validation failures leave the retained buffer and its previous values alone.
pub fn write_uniform_buffer(
    resource: &UniformBuffer,
    queue: &GPUQueue,
    value: Option<&Unknown>,
) -> Result<(), PackingError> {
    let bytes = try_pack(resource.layout, value)?;
    queue.write_buffer(resource.buffer, 0, bytes, None, None);
    Ok(())
}

pub fn create_vertex_buffer(
    device: &GPUDevice,
    vertices: &Float32Array,
    label: &str,
    capacity_bytes: Option<f64>,
) -> &'static GPUBuffer {
    let buffer = device.create_buffer(GPUBufferDescriptor {
        label: Some(label),
        size: capacity_bytes.unwrap_or_else(|| vertices.byte_length()),
        usage: f64::from(gpu_buffer_usage::VERTEX | gpu_buffer_usage::COPY_DST),
        mapped_at_creation: None,
    });
    device.queue().write_buffer(buffer, 0, vertices, None, None);
    buffer
}
