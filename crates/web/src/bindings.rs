//! Retained uniforms and reflected bind-group construction for the prism graph.
//! The landing shaders use group zero. WGSL reflection owns resource slots.
use crate::buffers::{UniformBuffer, create_packed_uniform_buffer, write_uniform_buffer};
use crate::packing::try_pack;
use crate::shader::ShaderSource;
use js::Unknown;
use react::webapi::{
    GPUBindGroup, GPUBindGroupDescriptor, GPUBindGroupEntry, GPUBindingResource, GPUBufferBinding,
    GPUDevice, GPUQueue, GPURenderPipeline,
};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

pub struct UniformInput<'a> {
    pub name: &'a str,
    pub value: Option<&'a Unknown>,
}
impl<'a> UniformInput<'a> {
    /// Shader reflection accepts heterogeneous parameter records. Erase only
    /// here, retaining Rust types everywhere that creates those records.
    pub fn new<T: 'static>(name: &'a str, value: T) -> Self {
        Self {
            name,
            value: js::any_of(value),
        }
    }
}
pub struct NamedResource<'a> {
    pub name: &'a str,
    pub resource: GPUBindingResource<'a>,
}
pub struct UniformBinding {
    pub name: String,
    pub slot: f64,
    pub resource: UniformBuffer,
}
pub struct ShaderBindings {
    pub source: &'static ShaderSource,
    pub uniforms: Vec<UniformBinding>,
}

/// Pack every input before allocating GPU buffers, so a later invalid block
/// cannot leak the buffers already allocated for an earlier block.
pub fn prepare_shader_bindings(
    device: &GPUDevice,
    source: &'static ShaderSource,
    inputs: &[UniformInput<'_>],
    label: &str,
) -> Result<ShaderBindings, String> {
    let reflected = source.reflection().bindings();
    if reflected.iter().any(|binding| binding.group() != 0.0) {
        return Err("The prism graph requires shader bindings in group zero.".into());
    }
    let mut packed = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        if inputs[..index]
            .iter()
            .any(|previous| previous.name == input.name)
        {
            return Err(format!("Duplicate uniform '{}'.", input.name));
        }
        let binding = reflected
            .iter()
            .find(|binding| binding.name() == input.name)
            .ok_or_else(|| format!("Unknown uniform '{}'.", input.name))?;
        if binding.address_space().as_deref() != Some("uniform") {
            return Err(format!("Binding '{}' is not a uniform buffer.", input.name));
        }
        let layout = binding
            .layout()
            .ok_or_else(|| format!("Uniform '{}' has no reflected layout.", input.name))?;
        let bytes = try_pack(layout, input.value).map_err(|error| format!("{error:?}"))?;
        packed.push((input.name, binding.binding(), layout, bytes));
    }
    let uniforms = packed
        .into_iter()
        .map(|(name, slot, layout, bytes)| UniformBinding {
            name: name.into(),
            slot,
            resource: create_packed_uniform_buffer(
                device,
                layout,
                bytes,
                &format!("{label}.{name}"),
            ),
        })
        .collect();
    Ok(ShaderBindings { source, uniforms })
}

impl ShaderBindings {
    /// Texture views change during resize or mip baking; the uniform buffers
    /// stay retained. The caller keeps the resulting bind group while its
    /// textures are unchanged, and only updates the uniform bytes per frame.
    pub fn bind(
        &self,
        device: &GPUDevice,
        pipeline: &GPURenderPipeline,
        resources: &[NamedResource<'_>],
        label: &str,
    ) -> Result<&'static GPUBindGroup, String> {
        let reflected = self.source.reflection().bindings();
        let mut entries: Vec<GPUBindGroupEntry<'_>> = self
            .uniforms
            .iter()
            .map(|uniform| GPUBindGroupEntry {
                binding: uniform.slot,
                resource: GPUBufferBinding {
                    buffer: uniform.resource.buffer,
                    offset: None,
                    size: None,
                }
                .into(),
            })
            .collect();
        for named in resources {
            let binding = reflected
                .iter()
                .find(|binding| binding.name() == named.name)
                .ok_or_else(|| format!("Unknown resource '{}'.", named.name))?;
            let slot = binding.binding();
            if entries.iter().any(|entry| entry.binding == slot) {
                return Err(format!("Duplicate resource '{}'.", named.name));
            }
            entries.push(GPUBindGroupEntry {
                binding: slot,
                resource: named.resource,
            });
        }
        if let Some(missing) = reflected.iter().find(|binding| {
            !entries
                .iter()
                .any(|entry| entry.binding == binding.binding())
        }) {
            return Err(format!("Missing resource '{}'.", missing.name()));
        }
        Ok(device.create_bind_group(GPUBindGroupDescriptor {
            label: Some(label),
            layout: pipeline.get_bind_group_layout(0),
            entries: &entries,
        }))
    }

    pub fn write(
        &self,
        queue: &GPUQueue,
        name: &str,
        value: Option<&Unknown>,
    ) -> Result<(), String> {
        let uniform = self
            .uniforms
            .iter()
            .find(|uniform| uniform.name == name)
            .ok_or_else(|| format!("Unknown uniform '{name}'."))?;
        write_uniform_buffer(&uniform.resource, queue, value).map_err(|error| format!("{error:?}"))
    }
    pub fn write_value<T: 'static>(
        &self,
        queue: &GPUQueue,
        name: &str,
        value: T,
    ) -> Result<(), String> {
        self.write(queue, name, js::any_of(value))
    }
    pub fn destroy(&self) {
        for uniform in &self.uniforms {
            uniform.resource.buffer.destroy();
        }
    }
}

/// A compiled native pipeline with its reflected resources and retained uniforms.
/// Pipeline creation belongs to each draw kind; binding ownership lives here.
pub struct BoundPipeline {
    pub pipeline: &'static GPURenderPipeline,
    pub values: ShaderBindings,
    pub group: &'static GPUBindGroup,
}
impl BoundPipeline {
    pub fn prepare(
        device: &GPUDevice,
        source: &'static ShaderSource,
        pipeline: &'static GPURenderPipeline,
        inputs: &[UniformInput<'_>],
        resources: &[NamedResource<'_>],
        label: &str,
    ) -> Result<Self, String> {
        let values = prepare_shader_bindings(device, source, inputs, label)?;
        // A native API can throw after uniform allocation. Keep cleanup at the
        // binding owner so every graph has the same exception guarantee.
        let bound = match catch_unwind(AssertUnwindSafe(|| {
            values.bind(device, pipeline, resources, label)
        })) {
            Ok(result) => result,
            Err(error) => {
                values.destroy();
                resume_unwind(error);
            }
        };
        let group = match bound {
            Ok(group) => group,
            Err(error) => {
                values.destroy();
                return Err(error);
            }
        };
        Ok(Self {
            pipeline,
            values,
            group,
        })
    }
    pub fn rebind(
        &mut self,
        device: &GPUDevice,
        resources: &[NamedResource<'_>],
        label: &str,
    ) -> Result<(), String> {
        self.group = self.values.bind(device, self.pipeline, resources, label)?;
        Ok(())
    }
    pub fn destroy(&self) {
        self.values.destroy();
    }
}
