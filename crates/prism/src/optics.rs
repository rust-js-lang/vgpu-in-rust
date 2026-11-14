//! CPU optics used directly by the browser scene.
//!
//! Snell refraction, Fresnel transmission and total internal reflection are
//! solved here for the two boundaries of every wavelength band. The light mesh
//! turns those deterministic paths into the vertices the GPU rasterizes.
//! Source: components/prism-background/scene/optics.ts.

use crate::types::{PRISM_WAVELENGTH_MAX, PRISM_WAVELENGTH_MIN, Triangle, Vec2, Vec3};
use crate::{clamp, number_max, number_min};

/// Keeps a ray from immediately re-hitting the surface it just left.
pub const SURFACE_EPSILON: f64 = 1e-4;

fn add(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] + b[0], a[1] + b[1]]
}
fn sub(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] - b[0], a[1] - b[1]]
}
fn scale(a: Vec2, k: f64) -> Vec2 {
    [a[0] * k, a[1] * k]
}
fn dot(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn length(a: Vec2) -> f64 {
    a[0].hypot(a[1])
}
fn normalize(a: Vec2) -> Vec2 {
    scale(a, 1.0 / length(a))
}
/// z of the 3D cross product: positive when b is left of a.
fn cross(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}

/// Signed area doubled; positive for counter-clockwise winding.
pub fn triangle_winding(triangle: Triangle) -> f64 {
    cross(sub(triangle.b, triangle.a), sub(triangle.c, triangle.a))
}

/// True when the point is on the inner side of all three edges.
pub fn inside_triangle(triangle: Triangle, point: Vec2) -> bool {
    let sign = if triangle_winding(triangle) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let ab = cross(sub(triangle.b, triangle.a), sub(point, triangle.a)) * sign;
    let bc = cross(sub(triangle.c, triangle.b), sub(point, triangle.b)) * sign;
    let ca = cross(sub(triangle.a, triangle.c), sub(point, triangle.c)) * sign;
    ab >= 0.0 && bc >= 0.0 && ca >= 0.0
}

/// Cauchy's empirical dispersion law, wavelength in nanometres.
pub fn ior_at(wavelength_nm: f64, base: f64, strength: f64) -> f64 {
    let micrometres = wavelength_nm * 1e-3;
    base + strength / (micrometres * micrometres)
}

#[derive(Clone, Copy, Debug)]
pub struct EdgeHit {
    /// Distance along the ray.
    pub t: f64,
    /// Unit normal pointing out of the triangle.
    pub normal: Vec2,
    /// Triangle edge index: a-b, b-c or c-a.
    pub edge: usize,
}

/// Nearest crossing of the triangle boundary strictly beyond min_t.
///
/// Works from inside and outside: the caller decides what a hit means from
/// the sign of dot(direction, normal).
pub fn intersect_triangle(
    triangle: Triangle,
    origin: Vec2,
    direction: Vec2,
    min_t: f64,
) -> Option<EdgeHit> {
    let vertices = [triangle.a, triangle.b, triangle.c];
    let mut best: Option<EdgeHit> = None;
    for index in 0..3 {
        let edge_start = vertices[index];
        let edge_end = vertices[(index + 1) % 3];
        let edge = sub(edge_end, edge_start);
        let denominator = cross(direction, edge);
        if denominator == 0.0 {
            continue;
        }
        let offset = sub(edge_start, origin);
        let t = cross(offset, edge) / denominator;
        let s = cross(offset, direction) / denominator;
        if t <= min_t || s < 0.0 || s > 1.0 {
            continue;
        }
        if best.is_some_and(|best| best.t <= t) {
            continue;
        }
        // Counter-clockwise winding puts the interior left of every edge;
        // rotating clockwise points out of the triangle.
        best = Some(EdgeHit {
            t,
            normal: normalize([edge[1], -edge[0]]),
            edge: index,
        });
    }
    best
}

/// Snell's law in the plane. The normal faces the incident medium and eta is
/// incident IOR over transmitted IOR. None means total internal reflection.
pub fn refract(incident: Vec2, normal: Vec2, eta: f64) -> Option<Vec2> {
    let cos_incident = -dot(incident, normal);
    let sin_transmitted_squared = eta * eta * (1.0 - cos_incident * cos_incident);
    if sin_transmitted_squared > 1.0 {
        return None;
    }
    let cos_transmitted = (1.0 - sin_transmitted_squared).sqrt();
    Some(add(
        scale(incident, eta),
        scale(normal, eta * cos_incident - cos_transmitted),
    ))
}

pub fn reflect(incident: Vec2, normal: Vec2) -> Vec2 {
    sub(incident, scale(normal, 2.0 * dot(incident, normal)))
}

/// Fraction of unpolarized light transmitted by an ideal dielectric boundary.
/// The exact Fresnel equations average both polarizations; TIR returns zero.
pub fn fresnel_transmittance(
    incident: Vec2,
    normal: Vec2,
    incident_ior: f64,
    transmitted_ior: f64,
) -> f64 {
    let cos_incident = clamp(-dot(incident, normal), 0.0, 1.0);
    let eta = incident_ior / transmitted_ior;
    let sin_transmitted_squared = eta * eta * (1.0 - cos_incident * cos_incident);
    if sin_transmitted_squared >= 1.0 {
        return 0.0;
    }
    let cos_transmitted = (1.0 - sin_transmitted_squared).sqrt();
    let s_numerator = incident_ior * cos_incident - transmitted_ior * cos_transmitted;
    let s_denominator = incident_ior * cos_incident + transmitted_ior * cos_transmitted;
    let p_numerator = incident_ior * cos_transmitted - transmitted_ior * cos_incident;
    let p_denominator = incident_ior * cos_transmitted + transmitted_ior * cos_incident;
    let reflectance =
        0.5 * ((s_numerator / s_denominator).powi(2) + (p_numerator / p_denominator).powi(2));
    1.0 - reflectance
}

#[derive(Clone, Copy, Debug)]
pub struct PrismPath {
    /// Where the ray left the glass.
    pub origin: Vec2,
    /// Unit direction it left with.
    pub direction: Vec2,
    /// Internal reflections taken before it got out.
    pub bounces: usize,
}

#[derive(Clone, Debug)]
pub struct DetailedPrismPath {
    pub origin: Vec2,
    pub direction: Vec2,
    pub bounces: usize,
    /// Entry, reflection points and final exit, in traversal order.
    pub points: Vec<Vec2>,
    /// Edge index for each point.
    pub edges: Vec<usize>,
    /// Fresnel transmission accumulated at entry and final exit.
    pub transmission: f64,
    /// Fresnel transmission at the air-to-glass interface only.
    pub entry_transmission: f64,
}

/// Refract a ray through the prism, bouncing internally when total internal
/// reflection prevents it from leaving, until it escapes or exhausts its budget.
pub fn trace_prism(
    triangle: Triangle,
    origin: Vec2,
    direction: Vec2,
    ior: f64,
    max_bounces: usize,
) -> Option<PrismPath> {
    let path = trace_prism_detailed(triangle, origin, direction, ior, max_bounces)?;
    Some(PrismPath {
        origin: path.origin,
        direction: path.direction,
        bounces: path.bounces,
    })
}

/// Detailed forward path used to turn a finite beam into renderable ribbons.
pub fn trace_prism_detailed(
    triangle: Triangle,
    origin: Vec2,
    direction: Vec2,
    ior: f64,
    max_bounces: usize,
) -> Option<DetailedPrismPath> {
    let entry = intersect_triangle(triangle, origin, direction, SURFACE_EPSILON)?;
    // A ray first meeting a boundary from behind started inside the glass.
    if dot(direction, entry.normal) >= 0.0 {
        return None;
    }
    let mut position = add(origin, scale(direction, entry.t));
    let mut inside = refract(direction, entry.normal, 1.0 / ior)?;
    let mut points = vec![position];
    let mut edges = vec![entry.edge];
    let entry_transmission = fresnel_transmittance(direction, entry.normal, 1.0, ior);
    let mut transmission = entry_transmission;
    for bounces in 0..=max_bounces {
        let exit = intersect_triangle(triangle, position, inside, SURFACE_EPSILON)?;
        position = add(position, scale(inside, exit.t));
        points.push(position);
        edges.push(exit.edge);
        if let Some(transmitted) = refract(inside, scale(exit.normal, -1.0), ior) {
            transmission *= fresnel_transmittance(inside, scale(exit.normal, -1.0), ior, 1.0);
            return Some(DetailedPrismPath {
                origin: position,
                direction: normalize(transmitted),
                bounces,
                points,
                edges,
                transmission,
                entry_transmission,
            });
        }
        inside = reflect(inside, exit.normal);
    }
    None
}

// Analytic CIE 1931 color matching approximations: Wyman, Sloan, Shirley, JCGT 2013.
fn cie_x(wavelength_nm: f64) -> f64 {
    let t1 = (wavelength_nm - 442.0)
        * if wavelength_nm < 442.0 {
            0.0624
        } else {
            0.0374
        };
    let t2 = (wavelength_nm - 599.8)
        * if wavelength_nm < 599.8 {
            0.0264
        } else {
            0.0323
        };
    let t3 = (wavelength_nm - 501.1)
        * if wavelength_nm < 501.1 {
            0.0490
        } else {
            0.0382
        };
    0.362 * (-0.5 * t1 * t1).exp() + 1.056 * (-0.5 * t2 * t2).exp() - 0.065 * (-0.5 * t3 * t3).exp()
}
fn cie_y(wavelength_nm: f64) -> f64 {
    let t1 = (wavelength_nm - 568.8)
        * if wavelength_nm < 568.8 {
            0.0213
        } else {
            0.0247
        };
    let t2 = (wavelength_nm - 530.9)
        * if wavelength_nm < 530.9 {
            0.0613
        } else {
            0.0322
        };
    0.821 * (-0.5 * t1 * t1).exp() + 0.286 * (-0.5 * t2 * t2).exp()
}
fn cie_z(wavelength_nm: f64) -> f64 {
    let t1 = (wavelength_nm - 437.0)
        * if wavelength_nm < 437.0 {
            0.0845
        } else {
            0.0278
        };
    let t2 = (wavelength_nm - 459.0)
        * if wavelength_nm < 459.0 {
            0.0385
        } else {
            0.0725
        };
    1.217 * (-0.5 * t1 * t1).exp() + 0.681 * (-0.5 * t2 * t2).exp()
}

/// Linear sRGB for a single wavelength. Spectral colors outside the sRGB gamut
/// have negative matrix channels; clamping preserves the original hue ordering.
pub fn wavelength_to_linear_rgb(wavelength_nm: f64) -> Vec3 {
    let x = cie_x(wavelength_nm);
    let y = cie_y(wavelength_nm);
    let z = cie_z(wavelength_nm);
    [
        number_max(0.0, 3.2406 * x - 1.5372 * y - 0.4986 * z),
        number_max(0.0, -0.9689 * x + 1.8758 * y + 0.0415 * z),
        number_max(0.0, 0.0557 * x - 0.2040 * y + 1.0570 * z),
    ]
}

/// CIE standard illuminant D65 at 10 nm intervals from 400 through 700 nm.
/// Linear interpolation suffices for the mesh's 128 wavelength vertices.
/// Source: CIE.DS.hjfjmt59.
const D65_SPECTRAL_POWER: [f64; 31] = [
    82.7549, 91.486, 93.4318, 86.6823, 104.865, 117.008, 117.812, 114.861, 115.923, 108.811,
    109.354, 107.802, 104.79, 107.689, 104.405, 104.046, 100.0, 96.3342, 95.788, 88.6856, 90.0062,
    89.5991, 87.6987, 83.2886, 83.6992, 80.0268, 80.2146, 82.2778, 78.2842, 69.7213, 71.6091,
];
/// Peak of D65(lambda) * CIE y-bar(lambda) across the rendered range.
const D65_PHOTOPIC_PEAK: f64 = 1.0347;
/// One photographic shoulder shared by every wavelength.
const SPECTRAL_EXPOSURE: f64 = 4.5;
/// Global adaptation makes the complete gamut-mapped D65 spectrum white.
const SPECTRAL_WHITE_BALANCE: Vec3 = [1.1868, 1.0, 2.2495];

fn d65_spectral_power(wavelength_nm: f64) -> f64 {
    let coordinate = clamp(
        (wavelength_nm - PRISM_WAVELENGTH_MIN) / 10.0,
        0.0,
        (D65_SPECTRAL_POWER.len() - 1) as f64,
    );
    // JS's undefined array lookup propagates NaN; indexing Rust with NaN as
    // usize would select zero instead. Preserve the original numeric result.
    if coordinate.is_nan() {
        return f64::NAN;
    }
    let lower = number_min((D65_SPECTRAL_POWER.len() - 2) as f64, coordinate.floor()) as usize;
    let fraction = coordinate - lower as f64;
    (D65_SPECTRAL_POWER[lower] * (1.0 - fraction) + D65_SPECTRAL_POWER[lower + 1] * fraction)
        / 100.0
}

/// Positive linear-sRGB radiance under daylight D65.
///
/// Shift toward neutral before one shared exposure, rather than giving each
/// wavelength equal peak energy. Integrating the spectrum reconstructs white.
/// Fresnel losses are already folded into each mesh vertex's intensity.
pub fn wavelength_to_beam_rgb(wavelength_nm: f64) -> Vec3 {
    let wavelength = clamp(wavelength_nm, PRISM_WAVELENGTH_MIN, PRISM_WAVELENGTH_MAX);
    let x = cie_x(wavelength);
    let y = cie_y(wavelength);
    let z = cie_z(wavelength);
    let linear_rgb = [
        3.2406 * x - 1.5372 * y - 0.4986 * z,
        -0.9689 * x + 1.8758 * y + 0.0415 * z,
        0.0557 * x - 0.2040 * y + 1.0570 * z,
    ];
    let neutral_offset = number_min(
        number_min(number_min(0.0, linear_rgb[0]), linear_rgb[1]),
        linear_rgb[2],
    );
    let positive_rgb = [
        linear_rgb[0] - neutral_offset,
        linear_rgb[1] - neutral_offset,
        linear_rgb[2] - neutral_offset,
    ];
    let hue_peak = number_max(
        number_max(
            number_max(positive_rgb[0], positive_rgb[1]),
            positive_rgb[2],
        ),
        f64::EPSILON,
    );
    let relative_photopic_power = d65_spectral_power(wavelength) * y / D65_PHOTOPIC_PEAK;
    let display_power = (1.0 - (-SPECTRAL_EXPOSURE * relative_photopic_power).exp())
        / (1.0 - (-SPECTRAL_EXPOSURE).exp());
    [
        (positive_rgb[0] / hue_peak) * display_power * SPECTRAL_WHITE_BALANCE[0],
        (positive_rgb[1] / hue_peak) * display_power * SPECTRAL_WHITE_BALANCE[1],
        (positive_rgb[2] / hue_peak) * display_power * SPECTRAL_WHITE_BALANCE[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRIANGLE: Triangle = Triangle {
        a: [0.0, 1.0],
        b: [-1.0, -1.0],
        c: [1.0, -1.0],
    };

    #[test]
    fn snell_fresnel_and_total_internal_reflection() {
        assert_eq!(
            refract([0.0, -1.0], [0.0, 1.0], 1.0 / 1.5),
            Some([0.0, -1.0])
        );
        assert!((fresnel_transmittance([0.0, -1.0], [0.0, 1.0], 1.0, 1.5) - 0.96).abs() < 1e-12);
        assert_eq!(refract([0.8, -0.6], [0.0, 1.0], 1.5), None);
        assert_eq!(
            fresnel_transmittance([0.8, -0.6], [0.0, 1.0], 1.5, 1.0),
            0.0
        );
        assert_eq!(reflect([0.8, -0.6], [0.0, 1.0]), [0.8, 0.6]);
    }

    #[test]
    fn ray_path_exits_glass_and_rejects_rays_starting_inside() {
        assert!(inside_triangle(TRIANGLE, [0.0, 0.0]));
        assert!(!inside_triangle(TRIANGLE, [2.0, 0.0]));
        assert!(trace_prism(TRIANGLE, [0.0, 0.0], [1.0, 0.0], 1.5, 3).is_none());
        let path = trace_prism_detailed(TRIANGLE, [2.0, 0.0], [-1.0, 0.0], 1.3, 3).unwrap();
        assert_eq!(path.points.len(), path.edges.len());
        assert_eq!(path.points.len(), path.bounces + 2);
        assert!(path.transmission > 0.0 && path.transmission <= path.entry_transmission);
        assert!((length(path.direction) - 1.0).abs() < 1e-12);
        let outside = add(path.origin, scale(path.direction, 0.01));
        assert!(!inside_triangle(TRIANGLE, outside));
    }

    #[test]
    fn spectral_mapping_preserves_hue_order_and_invalid_input() {
        let blue = wavelength_to_beam_rgb(450.0);
        let green = wavelength_to_beam_rgb(550.0);
        let red = wavelength_to_beam_rgb(650.0);
        assert!(blue[2] > blue[0] && blue[2] > blue[1]);
        assert!(green[1] > green[0] && green[1] > green[2]);
        assert!(red[0] > red[1] && red[0] > red[2]);
        assert_eq!(wavelength_to_beam_rgb(350.0), wavelength_to_beam_rgb(400.0));
        assert_eq!(wavelength_to_beam_rgb(800.0), wavelength_to_beam_rgb(700.0));
        assert!(wavelength_to_beam_rgb(f64::NAN).iter().all(|x| x.is_nan()));
        assert!(ior_at(400.0, 1.3, 0.01) > ior_at(700.0, 1.3, 0.01));
    }
}
