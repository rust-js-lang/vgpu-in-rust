//! Deterministic finite-width spectral sheets from scene/light-mesh.ts.
//! Wavelength neighbors interpolate without bands; additive sheets integrate
//! across the collimated beam. Packed doubles convert once at the GPU boundary.
use crate::optics::{
    DetailedPrismPath, intersect_triangle, ior_at, trace_prism_detailed, wavelength_to_beam_rgb,
};
use crate::types::{
    CollimatedLight, DEFAULT_LIGHT_EDGE_FALLOFF, DispersionPreset, PRISM_BEAM_SLICES,
    PRISM_LIGHT_EXPOSURE, PRISM_MAX_INTERNAL_BOUNCES, PRISM_SPECTRAL_SAMPLES, PRISM_TRIANGLE,
    PRISM_WAVELENGTH_MAX, PRISM_WAVELENGTH_MIN, Triangle, Vec2,
};
use crate::{clamp, number_max, number_min};

pub const LIGHT_VERTEX_FLOATS: usize = 3;
pub const LIGHT_VERTEX_STRIDE: usize = LIGHT_VERTEX_FLOATS * size_of::<f32>();
const VERTICES_PER_QUAD: usize = 6;
pub const LIGHT_INTERNAL_SEGMENTS: usize = PRISM_MAX_INTERNAL_BOUNCES + 1;
const DENSITY_MEASURE_DISTANCE: f64 = 1.0;
pub const INPUT_BEAM_RADIANCE: f64 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightMeshLayout {
    pub samples: usize,
    pub beam_slices: usize,
    pub white_quads: usize,
    pub internal_quads: usize,
    pub white_vertices: usize,
    pub internal_first_vertex: usize,
    pub internal_vertices: usize,
    pub outgoing_first_vertex: usize,
    pub outgoing_vertices: usize,
    pub vertex_count: usize,
}

pub const fn light_mesh_layout(samples: usize, beam_slices: usize) -> LightMeshLayout {
    let samples = if samples < 2 { 2 } else { samples };
    let beam_slices = if beam_slices < 1 { 1 } else { beam_slices };
    let white_quads = beam_slices;
    let internal_quads = samples * beam_slices * LIGHT_INTERNAL_SEGMENTS;
    let white_vertices = white_quads * VERTICES_PER_QUAD;
    let internal_vertices = internal_quads * VERTICES_PER_QUAD;
    let outgoing_vertices = (samples - 1) * beam_slices * VERTICES_PER_QUAD;
    LightMeshLayout {
        samples,
        beam_slices,
        white_quads,
        internal_quads,
        white_vertices,
        internal_first_vertex: white_vertices,
        internal_vertices,
        outgoing_first_vertex: white_vertices + internal_vertices,
        outgoing_vertices,
        vertex_count: white_vertices + internal_vertices + outgoing_vertices,
    }
}

pub const HIGH_LIGHT_MESH_LAYOUT: LightMeshLayout =
    light_mesh_layout(PRISM_SPECTRAL_SAMPLES, PRISM_BEAM_SLICES);

pub fn light_vertex_count(samples: usize, beam_slices: usize) -> usize {
    (beam_slices
        + samples * beam_slices * LIGHT_INTERNAL_SEGMENTS
        + samples.saturating_sub(1).max(1) * beam_slices)
        * VERTICES_PER_QUAD
}

#[derive(Clone, Debug)]
pub struct SpectralBand {
    pub wavelength: f64,
    pub lower: DetailedPrismPath,
    pub upper: DetailedPrismPath,
    pub output_width: f64,
    pub transmission: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightMeshStats {
    pub samples: usize,
    pub beam_slices: usize,
    pub valid_bands: usize,
    pub rejected_topology: usize,
    pub min_output_width: f64,
    pub max_output_width: f64,
    /// Integrated scalar flux, useful for checking subdivision invariance.
    pub total_flux: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightVertexMetadata {
    pub wavelength: f64,
    pub profile: f64,
    pub travel: f64,
    /// -1 identifies the white input beam.
    pub spectral_index: f64,
}

pub struct LightMeshOptions {
    pub light: CollimatedLight,
    pub dispersion: DispersionPreset,
    pub wall_half_extent: Vec2,
    pub triangle: Option<Triangle>,
    pub samples: Option<usize>,
    pub beam_slices: Option<usize>,
    pub exposure: Option<f64>,
    pub edge_falloff: Option<f64>,
}

impl LightMeshOptions {
    pub fn layout(&self) -> LightMeshLayout {
        light_mesh_layout(
            self.samples.unwrap_or(PRISM_SPECTRAL_SAMPLES),
            self.beam_slices.unwrap_or(PRISM_BEAM_SLICES),
        )
    }

    pub fn new(
        light: CollimatedLight,
        dispersion: DispersionPreset,
        wall_half_extent: Vec2,
    ) -> Self {
        Self {
            light,
            dispersion,
            wall_half_extent,
            triangle: None,
            samples: None,
            beam_slices: None,
            exposure: None,
            edge_falloff: None,
        }
    }
}

struct SpectralNode {
    wavelength: f64,
    paths: Vec<Option<DetailedPrismPath>>,
    boundary_paths: Vec<Option<DetailedPrismPath>>,
}

struct WhiteBoundary {
    profile: f64,
    entry: Option<Vec2>,
    wall: Option<[Vec2; 2]>,
}

const UPPER_QUAD_VERTEX: [bool; 6] = [false, true, true, false, true, false];
const END_QUAD_VERTEX: [bool; 6] = [false, false, true, false, true, true];
fn add(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn scale(a: Vec2, amount: f64) -> Vec2 {
    [a[0] * amount, a[1] * amount]
}
fn cross(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn dot(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn normalize(a: Vec2) -> Vec2 {
    let magnitude = a[0].hypot(a[1]);
    let magnitude = if magnitude == 0.0 || magnitude.is_nan() {
        1.0
    } else {
        magnitude
    };
    [a[0] / magnitude, a[1] / magnitude]
}

pub fn beam_profile_origin(light: CollimatedLight, profile: f64) -> Vec2 {
    let perpendicular = [-light.direction[1], light.direction[0]];
    add(
        light.center,
        scale(
            perpendicular,
            light.beam_half_width * clamp(profile, -1.0, 1.0),
        ),
    )
}

pub fn beam_boundary_origins(light: CollimatedLight) -> [Vec2; 2] {
    [
        beam_profile_origin(light, -1.0),
        beam_profile_origin(light, 1.0),
    ]
}

pub fn ray_to_wall_boundary(origin: Vec2, direction: Vec2, half_extent: Vec2) -> Vec2 {
    let mut nearest = f64::INFINITY;
    for axis in 0..2 {
        let component = direction[axis];
        if component.abs() < 1e-8 {
            continue;
        }
        for side in [-half_extent[axis], half_extent[axis]] {
            let distance = (side - origin[axis]) / component;
            if distance <= 0.0 || distance >= nearest {
                continue;
            }
            let other = 1 - axis;
            let other_coordinate = origin[other] + direction[other] * distance;
            if other_coordinate.abs() <= half_extent[other] + 1e-6 {
                nearest = distance;
            }
        }
    }
    if nearest.is_finite() {
        add(origin, scale(direction, nearest))
    } else {
        origin
    }
}

pub fn line_through_wall(origin: Vec2, direction: Vec2, half_extent: Vec2) -> Option<[Vec2; 2]> {
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    for axis in 0..2 {
        let component = direction[axis];
        let coordinate = origin[axis];
        let extent = half_extent[axis];
        if component.abs() < 1e-8 {
            if coordinate.abs() > extent {
                return None;
            }
            continue;
        }
        let first = (-extent - coordinate) / component;
        let second = (extent - coordinate) / component;
        near = number_max(near, number_min(first, second));
        far = number_min(far, number_max(first, second));
        if near > far {
            return None;
        }
    }
    near = number_max(0.0, near);
    if !near.is_finite() || !far.is_finite() || far < near {
        return None;
    }
    Some([
        add(origin, scale(direction, near)),
        add(origin, scale(direction, far)),
    ])
}

/// Exact forward strip/triangle overlap, including grazing beams that cross a
/// face only between two sampled boundaries.
pub fn beam_intersects_triangle(triangle: Triangle, light: CollimatedLight) -> bool {
    let perpendicular = [-light.direction[1], light.direction[0]];
    let mut polygon: Vec<Vec2> = [triangle.a, triangle.b, triangle.c]
        .iter()
        .map(|&point| {
            let offset = sub(point, light.center);
            [dot(offset, light.direction), dot(offset, perpendicular)]
        })
        .collect();
    // Half-plane clipping in beam-local coordinates.
    for plane in 0..3 {
        let inside = |point: Vec2| match plane {
            0 => point[0],
            1 => point[1] + light.beam_half_width,
            _ => light.beam_half_width - point[1],
        };
        let input = polygon;
        polygon = Vec::new();
        for index in 0..input.len() {
            let start = input[index];
            let end = input[(index + 1) % input.len()];
            let start_distance = inside(start);
            let end_distance = inside(end);
            let start_inside = start_distance >= 0.0;
            let end_inside = end_distance >= 0.0;
            if start_inside {
                polygon.push(start);
            }
            if start_inside == end_inside {
                continue;
            }
            let amount = start_distance / (start_distance - end_distance);
            polygon.push([
                start[0] + (end[0] - start[0]) * amount,
                start[1] + (end[1] - start[1]) * amount,
            ]);
        }
        if polygon.is_empty() {
            return false;
        }
    }
    !polygon.is_empty()
}

fn matching_topology(a: &DetailedPrismPath, b: &DetailedPrismPath) -> bool {
    a.edges == b.edges
}

pub fn trace_spectral_band(
    triangle: Triangle,
    light: CollimatedLight,
    dispersion: DispersionPreset,
    wavelength: f64,
) -> Option<SpectralBand> {
    let ior = ior_at(wavelength, dispersion.base, dispersion.strength);
    let [lower_origin, upper_origin] = beam_boundary_origins(light);
    let lower = trace_prism_detailed(
        triangle,
        lower_origin,
        light.direction,
        ior,
        PRISM_MAX_INTERNAL_BOUNCES,
    )?;
    let upper = trace_prism_detailed(
        triangle,
        upper_origin,
        light.direction,
        ior,
        PRISM_MAX_INTERNAL_BOUNCES,
    )?;
    if !matching_topology(&lower, &upper) {
        return None;
    }
    let direction = normalize(add(lower.direction, upper.direction));
    let output_width = cross(sub(upper.origin, lower.origin), direction).abs();
    if output_width <= 1e-5 {
        return None;
    }
    let transmission = (lower.transmission + upper.transmission) * 0.5;
    Some(SpectralBand {
        wavelength,
        lower,
        upper,
        output_width,
        transmission,
    })
}

fn trace_profile_path(
    triangle: Triangle,
    light: CollimatedLight,
    dispersion: DispersionPreset,
    wavelength: f64,
    profile: f64,
) -> Option<DetailedPrismPath> {
    trace_prism_detailed(
        triangle,
        beam_profile_origin(light, profile),
        light.direction,
        ior_at(wavelength, dispersion.base, dispersion.strength),
        PRISM_MAX_INTERNAL_BOUNCES,
    )
}

/// Four corners ordered lower-start, upper-start, lower-end, upper-end. The
/// same two-triangle topology carries either longitudinal or spectral energy.
fn push_quad(output: &mut Vec<f64>, points: [Vec2; 4], intensity: [f64; 4]) {
    for index in [0, 1, 3, 0, 3, 2] {
        let point = points[index];
        output.extend_from_slice(&[point[0], point[1], intensity[index]]);
    }
}
fn push_empty_quad(output: &mut Vec<f64>) {
    push_quad(output, [[0.0; 2]; 4], [-1.0; 4]);
}

/// Static attributes decoded from vertex_index by WGSL, never uploaded again.
pub fn light_vertex_metadata(
    vertex_index: usize,
    samples: usize,
    beam_slices: usize,
) -> LightVertexMetadata {
    let count = light_vertex_count(samples, beam_slices);
    assert!(
        vertex_index < count,
        "Light vertex index {vertex_index} is outside [0, {count})."
    );
    let quad = vertex_index / VERTICES_PER_QUAD;
    let corner = vertex_index % VERTICES_PER_QUAD;
    let upper = usize::from(UPPER_QUAD_VERTEX[corner]);
    if quad < beam_slices {
        return LightVertexMetadata {
            wavelength: -1.0,
            profile: f64::from((-1.0 + 2.0 * (quad + upper) as f64 / beam_slices as f64) as f32),
            travel: 0.0,
            spectral_index: -1.0,
        };
    }
    let internal_quads = samples * beam_slices * LIGHT_INTERNAL_SEGMENTS;
    let spectral_quad = quad - beam_slices;
    if spectral_quad < internal_quads {
        let quads_per_wavelength = beam_slices * LIGHT_INTERNAL_SEGMENTS;
        let spectral_index = spectral_quad / quads_per_wavelength;
        let slice = (spectral_quad % quads_per_wavelength) / LIGHT_INTERNAL_SEGMENTS;
        return LightVertexMetadata {
            wavelength: wavelength_at(spectral_index, samples),
            profile: f64::from((-1.0 + 2.0 * (slice + upper) as f64 / beam_slices as f64) as f32),
            travel: 0.0,
            spectral_index: spectral_index as f64,
        };
    }
    let outgoing_quad = spectral_quad - internal_quads;
    let spectral_index = outgoing_quad / beam_slices + upper;
    LightVertexMetadata {
        wavelength: wavelength_at(spectral_index, samples),
        profile: 0.0,
        travel: if END_QUAD_VERTEX[corner] { 1.0 } else { 0.0 },
        spectral_index: spectral_index as f64,
    }
}
fn wavelength_at(index: usize, samples: usize) -> f64 {
    f64::from(
        (PRISM_WAVELENGTH_MIN
            + (PRISM_WAVELENGTH_MAX - PRISM_WAVELENGTH_MIN) * (index as f64 / (samples - 1) as f64))
            as f32,
    )
}
fn profile_coordinates(slices: usize) -> Vec<f64> {
    (0..slices)
        .map(|index| -1.0 + 2.0 * (index as f64 + 0.5) / slices as f64)
        .collect()
}
fn normalized_profile_weights(profiles: &[f64], edge_falloff: f64) -> Vec<f64> {
    let weights: Vec<_> = profiles
        .iter()
        .map(|profile| {
            let edge = clamp((profile.abs() - 0.55) / 0.45, 0.0, 1.0);
            let smooth = edge * edge * (3.0 - 2.0 * edge);
            (-edge_falloff * profile * profile).exp() * (1.0 - smooth)
        })
        .collect();
    let sum = weights.iter().sum::<f64>();
    let sum = if sum == 0.0 || sum.is_nan() { 1.0 } else { sum };
    weights.iter().map(|weight| weight / sum).collect()
}
fn path_at(node: Option<&SpectralNode>, index: usize) -> Option<&DetailedPrismPath> {
    node?.paths.get(index)?.as_ref()
}
fn can_connect(a: Option<&SpectralNode>, b: Option<&SpectralNode>, profile_index: usize) -> bool {
    match (path_at(a, profile_index), path_at(b, profile_index)) {
        (Some(a), Some(b)) => matching_topology(a, b),
        _ => false,
    }
}
fn density_reference(path: &DetailedPrismPath) -> Vec2 {
    add(path.origin, scale(path.direction, DENSITY_MEASURE_DISTANCE))
}

/// Divide flux by the finite-difference spectral Jacobian so subdivision does
/// not change the integrated light energy.
fn spectral_density(
    nodes: &[Option<SpectralNode>],
    node_index: usize,
    profile_index: usize,
    exposure: f64,
    input_width: f64,
    profile_weight: f64,
) -> f64 {
    let Some(path) = path_at(nodes[node_index].as_ref(), profile_index) else {
        return 0.0;
    };
    let mut left = node_index as isize - 1;
    while left >= 0 && path_at(nodes[left as usize].as_ref(), profile_index).is_none() {
        left -= 1;
    }
    let mut right = node_index + 1;
    while right < nodes.len() && path_at(nodes[right].as_ref(), profile_index).is_none() {
        right += 1;
    }
    let left = if left < 0 { node_index } else { left as usize };
    let right = if right >= nodes.len() {
        node_index
    } else {
        right
    };
    if left == right {
        return 0.0;
    }
    let left_path =
        path_at(nodes[left].as_ref(), profile_index).expect("selected an existing path");
    let right_path =
        path_at(nodes[right].as_ref(), profile_index).expect("selected an existing path");
    if !matching_topology(left_path, right_path) {
        return 0.0;
    }
    let direction = normalize(add(left_path.direction, right_path.direction));
    let spectral_width = cross(
        sub(density_reference(right_path), density_reference(left_path)),
        direction,
    )
    .abs();
    let normalized_span = (right - left) as f64 / (nodes.len() - 1) as f64;
    let jacobian = spectral_width / normalized_span;
    exposure * input_width * profile_weight * path.transmission / number_max(jacobian, 1e-4)
}

/// Build into a reusable packed-number buffer. The web GPU boundary owns the
/// retained Float32 upload view and performs the original single bulk conversion.
pub fn build_light_mesh(options: &LightMeshOptions, output: &mut Vec<f64>) -> LightMeshStats {
    let triangle = options.triangle.unwrap_or(*PRISM_TRIANGLE);
    let layout = options.layout();
    let samples = layout.samples;
    let beam_slices = layout.beam_slices;
    let exposure = options.exposure.unwrap_or(PRISM_LIGHT_EXPOSURE);
    let edge_falloff = number_max(
        0.0,
        options.edge_falloff.unwrap_or(DEFAULT_LIGHT_EDGE_FALLOFF),
    );
    let expected_vertex_count = layout.vertex_count;
    let expected_floats = expected_vertex_count * LIGHT_VERTEX_FLOATS;
    output.clear();
    let input_width = options.light.beam_half_width * 2.0;
    let profiles = profile_coordinates(beam_slices);
    let profile_weights = normalized_profile_weights(&profiles, edge_falloff);
    // Slice white light as well: grazing cells may enter different faces, or
    // miss the triangle entirely, without invalidating neighboring cells.
    let boundary_profiles: Vec<_> = (0..=beam_slices)
        .map(|index| -1.0 + 2.0 * index as f64 / beam_slices as f64)
        .collect();
    let white_boundaries: Vec<_> = boundary_profiles
        .iter()
        .map(|&profile| {
            let origin = beam_profile_origin(options.light, profile);
            let hit = intersect_triangle(triangle, origin, options.light.direction, 1e-4);
            let entry = hit
                .filter(|hit| dot(options.light.direction, hit.normal) < 0.0)
                .map(|hit| add(origin, scale(options.light.direction, hit.t)));
            WhiteBoundary {
                profile,
                entry,
                wall: line_through_wall(origin, options.light.direction, options.wall_half_extent),
            }
        })
        .collect();
    let backwards = options.light.direction.map(|component| -component);
    for slice in 0..beam_slices {
        let lower = &white_boundaries[slice];
        let upper = &white_boundaries[slice + 1];
        if let (Some(lower_entry), Some(upper_entry)) = (lower.entry, upper.entry) {
            push_quad(
                output,
                [
                    ray_to_wall_boundary(lower_entry, backwards, options.wall_half_extent),
                    ray_to_wall_boundary(upper_entry, backwards, options.wall_half_extent),
                    lower_entry,
                    upper_entry,
                ],
                [0.0, 0.0, INPUT_BEAM_RADIANCE, INPUT_BEAM_RADIANCE],
            );
        } else {
            let center_profile = (lower.profile + upper.profile) * 0.5;
            let cell_light = CollimatedLight {
                center: beam_profile_origin(options.light, center_profile),
                direction: options.light.direction,
                beam_half_width: options.light.beam_half_width
                    * (upper.profile - lower.profile)
                    * 0.5,
            };
            if let (false, Some(lower_wall), Some(upper_wall)) = (
                beam_intersects_triangle(triangle, cell_light),
                lower.wall,
                upper.wall,
            ) {
                push_quad(
                    output,
                    [lower_wall[0], upper_wall[0], lower_wall[1], upper_wall[1]],
                    [INPUT_BEAM_RADIANCE; 4],
                );
            } else {
                push_empty_quad(output);
            }
        }
    }
    let mut nodes = Vec::with_capacity(samples);
    let mut min_output_width = f64::INFINITY;
    let mut max_output_width = 0.0;
    for index in 0..samples {
        let wavelength = PRISM_WAVELENGTH_MIN
            + (PRISM_WAVELENGTH_MAX - PRISM_WAVELENGTH_MIN) * (index as f64 / (samples - 1) as f64);
        let band = trace_spectral_band(triangle, options.light, options.dispersion, wavelength);
        let paths: Vec<_> = profiles
            .iter()
            .map(|&profile| {
                trace_profile_path(
                    triangle,
                    options.light,
                    options.dispersion,
                    wavelength,
                    profile,
                )
            })
            .collect();
        let boundary_paths: Vec<_> = boundary_profiles
            .iter()
            .map(|&profile| {
                trace_profile_path(
                    triangle,
                    options.light,
                    options.dispersion,
                    wavelength,
                    profile,
                )
            })
            .collect();
        if paths.iter().all(Option::is_none) && boundary_paths.iter().all(Option::is_none) {
            nodes.push(None);
            continue;
        }
        if let Some(band) = band {
            min_output_width = number_min(min_output_width, band.output_width);
            max_output_width = number_max(max_output_width, band.output_width);
        }
        nodes.push(Some(SpectralNode {
            wavelength,
            paths,
            boundary_paths,
        }));
    }
    let densities: Vec<Vec<f64>> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            profiles
                .iter()
                .enumerate()
                .map(|(profile_index, _)| {
                    if node.is_some() {
                        spectral_density(
                            &nodes,
                            index,
                            profile_index,
                            exposure,
                            input_width,
                            profile_weights[profile_index],
                        )
                    } else {
                        0.0
                    }
                })
                .collect()
        })
        .collect();
    // Normalize overlapping internal sheets back to the white input radiance.
    let internal_rgb_sum = nodes.iter().flatten().fold([0.0; 3], |sum, node| {
        let rgb = wavelength_to_beam_rgb(node.wavelength);
        [sum[0] + rgb[0], sum[1] + rgb[1], sum[2] + rgb[2]]
    });
    let internal_scale = INPUT_BEAM_RADIANCE
        / number_max(
            number_max(
                number_max(internal_rgb_sum[0], internal_rgb_sum[1]),
                internal_rgb_sum[2],
            ),
            1.0,
        );
    for node in &nodes {
        let Some(node) = node else {
            for _ in 0..beam_slices * LIGHT_INTERNAL_SEGMENTS {
                push_empty_quad(output);
            }
            continue;
        };
        for slice in 0..beam_slices {
            let lower = node.boundary_paths[slice].as_ref();
            let upper = node.boundary_paths[slice + 1].as_ref();
            let connected = match (lower, upper) {
                (Some(a), Some(b)) => matching_topology(a, b),
                _ => false,
            };
            let intensity = if connected {
                internal_scale
                    * (lower.expect("connected lower").entry_transmission
                        + upper.expect("connected upper").entry_transmission)
                    * 0.5
            } else {
                0.0
            };
            for segment in 0..LIGHT_INTERNAL_SEGMENTS {
                let corners = lower
                    .and_then(|a| Some((*a.points.get(segment)?, *a.points.get(segment + 1)?)))
                    .zip(upper.and_then(|b| {
                        Some((*b.points.get(segment)?, *b.points.get(segment + 1)?))
                    }));
                if let (true, Some(((lower_start, lower_end), (upper_start, upper_end)))) =
                    (connected, corners)
                {
                    push_quad(
                        output,
                        [lower_start, upper_start, lower_end, upper_end],
                        [intensity; 4],
                    );
                } else {
                    push_empty_quad(output);
                }
            }
        }
    }
    let mut total_flux = 0.0;
    for interval in 0..samples - 1 {
        let low = nodes[interval].as_ref();
        let high = nodes[interval + 1].as_ref();
        for (profile_index, &profile_weight) in profile_weights.iter().enumerate() {
            if !can_connect(low, high, profile_index) {
                push_empty_quad(output);
                continue;
            }
            let low_path = path_at(low, profile_index).expect("connected low path");
            let high_path = path_at(high, profile_index).expect("connected high path");
            let low_intensity = densities[interval][profile_index];
            let high_intensity = densities[interval + 1][profile_index];
            total_flux += exposure
                * input_width
                * profile_weight
                * (low_path.transmission + high_path.transmission)
                * 0.5
                / (samples - 1) as f64;
            push_quad(
                output,
                [
                    low_path.origin,
                    high_path.origin,
                    ray_to_wall_boundary(
                        low_path.origin,
                        low_path.direction,
                        options.wall_half_extent,
                    ),
                    ray_to_wall_boundary(
                        high_path.origin,
                        high_path.direction,
                        options.wall_half_extent,
                    ),
                ],
                [low_intensity, high_intensity, low_intensity, high_intensity],
            );
        }
    }
    let vertex_count = output.len() / LIGHT_VERTEX_FLOATS;
    assert!(
        output.len() == expected_floats && vertex_count == expected_vertex_count,
        "Light mesh wrote {vertex_count} vertices; expected {expected_vertex_count}."
    );
    let valid_bands = nodes.iter().flatten().count();
    LightMeshStats {
        samples,
        beam_slices,
        valid_bands,
        rejected_topology: samples - valid_bands,
        min_output_width: if min_output_width.is_finite() {
            min_output_width
        } else {
            0.0
        },
        max_output_width,
        total_flux,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{STYLIZED_DISPERSION, lamp_for_incidence};

    fn options() -> LightMeshOptions {
        LightMeshOptions::new(
            lamp_for_incidence(60.0, 0.025, 0.5),
            STYLIZED_DISPERSION,
            [2.8, 1.6],
        )
    }

    #[test]
    fn topology_ranges_partition_every_vertex_and_metadata_remains_finite() {
        for (samples, slices) in [(2, 1), (16, 4), (64, 12), (128, 24)] {
            let layout = light_mesh_layout(samples, slices);
            assert_eq!(
                layout.vertex_count,
                layout.white_vertices + layout.internal_vertices + layout.outgoing_vertices
            );
            assert_eq!(
                layout.outgoing_first_vertex,
                layout.internal_first_vertex + layout.internal_vertices
            );
            for vertex in 0..layout.vertex_count {
                let metadata = light_vertex_metadata(vertex, samples, slices);
                assert!(metadata.wavelength.is_finite());
                assert!((-1.0..=1.0).contains(&metadata.profile));
                assert!((0.0..=1.0).contains(&metadata.travel));
            }
        }
    }

    #[test]
    fn reusable_scratch_overwrites_old_geometry_and_never_uploads_nonfinite_vertices() {
        let mut options = options();
        options.samples = Some(16);
        options.beam_slices = Some(4);
        let mut scratch = vec![f64::NAN; 12];
        let first = build_light_mesh(&options, &mut scratch);
        assert_eq!(
            scratch.len(),
            options.layout().vertex_count * LIGHT_VERTEX_FLOATS
        );
        assert!(scratch.iter().all(|value| value.is_finite()));
        let original = scratch.clone();
        options.light = lamp_for_incidence(35.0, 0.1, 0.2);
        build_light_mesh(&options, &mut scratch);
        assert_ne!(scratch, original);
        options.light = lamp_for_incidence(60.0, 0.025, 0.5);
        assert_eq!(build_light_mesh(&options, &mut scratch), first);
        assert_eq!(scratch, original);
    }

    #[test]
    fn spectral_subdivision_preserves_integrated_flux() {
        let mut options = options();
        options.samples = Some(32);
        let mut scratch = Vec::new();
        let coarse = build_light_mesh(&options, &mut scratch);
        options.samples = Some(128);
        let fine = build_light_mesh(&options, &mut scratch);
        assert!((coarse.total_flux - fine.total_flux).abs() < 0.0005);
    }
}
