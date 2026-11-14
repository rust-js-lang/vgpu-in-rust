//! Retained GPU upload storage for the portable spectral sheet builder.
use js::{Float32Array, float32_array};
use vgpu_prism::light_mesh::{
    LIGHT_VERTEX_FLOATS, LightMeshOptions, LightMeshStats, build_light_mesh as build_packed_mesh,
};

pub struct LightMeshData {
    pub vertices: &'static Float32Array,
    pub vertex_count: usize,
    pub stats: LightMeshStats,
}

/// Mirrors the original target/scratch contract. Geometry uses reusable packed
/// numbers; this boundary performs one bulk Float32 conversion, with no per-frame
/// upload allocation when the caller supplies its retained target.
pub fn build_light_mesh(
    options: &LightMeshOptions,
    target: Option<&'static Float32Array>,
    scratch: Option<&mut Vec<f64>>,
) -> LightMeshData {
    let layout = options.layout();
    let expected_floats = layout.vertex_count * LIGHT_VERTEX_FLOATS;
    let vertices = match target {
        Some(target) if target.length() != expected_floats as f64 => {
            let message = format!(
                "Light mesh target has {} floats; expected {expected_floats}.",
                target.length()
            );
            js::throw(js::range_error::new(Some(&message), None).as_unknown())
        }
        Some(target) => target,
        None => float32_array::new(expected_floats as f64),
    };
    let mut local = Vec::new();
    let output = scratch.unwrap_or(&mut local);
    let stats = build_packed_mesh(options, output);
    vertices.set(output, None);
    LightMeshData {
        vertices,
        vertex_count: layout.vertex_count,
        stats,
    }
}

/// JS-facing decoder validates numbers before entering the integer topology core.
/// It preserves native RangeError, including NaN/infinity and fractional indices.
pub fn light_vertex_metadata(
    vertex_index: f64,
    samples: Option<usize>,
    beam_slices: Option<usize>,
) -> vgpu_prism::light_mesh::LightVertexMetadata {
    let samples = samples.unwrap_or(vgpu_prism::types::PRISM_SPECTRAL_SAMPLES);
    let beam_slices = beam_slices.unwrap_or(vgpu_prism::types::PRISM_BEAM_SLICES);
    let count = vgpu_prism::light_mesh::light_vertex_count(samples, beam_slices);
    if !vertex_index.is_finite()
        || vertex_index.fract() != 0.0
        || vertex_index < 0.0
        || vertex_index >= count as f64
    {
        let index = js::number::to_string(vertex_index, None);
        let message = format!("Light vertex index {index} is outside [0, {count}).");
        js::throw(js::range_error::new(Some(&message), None).as_unknown())
    }
    vgpu_prism::light_mesh::light_vertex_metadata(vertex_index as usize, samples, beam_slices)
}
