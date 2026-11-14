//! GPU ownership for the portable rounded prism mesh.
use crate::buffers::create_vertex_buffer;
use react::webapi::{
    GPUBindGroup, GPUBuffer, GPUBufferDescriptor, GPUDevice, GPURenderPassEncoder,
    GPURenderPipeline, gpu_buffer_usage,
};
use vgpu_prism::prism_mesh::{prism_mesh_data, prism_wireframe_indices};
use vgpu_prism::types::{PRISM_BACK_Z, PRISM_FRONT_Z, PRISM_TRIANGLE};

/// Shared by the two glass interfaces; debug topology is uploaded lazily.
#[derive(Clone, Copy)]
pub struct PrismBuffers {
    pub vertices: &'static GPUBuffer,
    pub indices: &'static GPUBuffer,
    pub index_count: usize,
}

pub fn create_prism_geometry(device: &GPUDevice, label: &str, wireframe: bool) -> PrismBuffers {
    let mesh = prism_mesh_data(*PRISM_TRIANGLE, PRISM_BACK_Z, PRISM_FRONT_Z);
    let vertices = js::float32_array::from(&mesh.vertices);
    let index_data = if wireframe {
        prism_wireframe_indices(&mesh.indices)
    } else {
        mesh.indices
    };
    let indices = js::uint16_array::from(&index_data);
    let vertices = create_vertex_buffer(device, vertices, &format!("{label}.vertices"), None);
    let index_buffer = device.create_buffer(GPUBufferDescriptor {
        label: Some(&format!("{label}.indices")),
        size: indices.byte_length(),
        usage: f64::from(gpu_buffer_usage::INDEX | gpu_buffer_usage::COPY_DST),
        mapped_at_creation: None,
    });
    device
        .queue()
        .write_buffer(index_buffer, 0, indices, None, None);
    PrismBuffers {
        vertices,
        indices: index_buffer,
        index_count: index_data.len(),
    }
}
impl PrismBuffers {
    pub fn destroy(&self) {
        self.vertices.destroy();
        self.indices.destroy();
    }
}

/// The graph supplies its load/resolve target so background, back interface,
/// internal light and front interface keep their original composition order.
pub fn encode_prism(
    pass: &GPURenderPassEncoder,
    pipeline: &GPURenderPipeline,
    bindings: &GPUBindGroup,
    mesh: &PrismBuffers,
) {
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, Some(bindings), None);
    pass.set_vertex_buffer(0, Some(mesh.vertices), None, None);
    pass.set_index_buffer(mesh.indices, "uint16", None, None);
    pass.draw_indexed(mesh.index_count as f64, None, None, None, None);
}
