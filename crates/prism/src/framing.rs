//! Responsive prism framing.
//! Source: components/prism-background/scene/framing.ts.

use std::borrow::Cow;

use crate::{clamp, number_max, number_min};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectLike {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

/// Top-origin viewport coordinates normalized to the canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedViewport {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Bounds in WebGPU normalized device coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NdcBounds {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// Uniform clip-space scale followed by a principal-point offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionFraming {
    pub scale: f64,
    pub offset: [f64; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectionDistanceFit {
    pub distance: f64,
    pub bounds: NdcBounds,
    pub framing: ProjectionFraming,
}

pub const IDENTITY_PROJECTION_FRAMING: ProjectionFraming = ProjectionFraming {
    scale: 1.0,
    offset: [0.0, 0.0],
};

/// Resolves a target element inside a canvas using CSS pixels.
///
/// Ratios make the result independent of DPR, while intersecting both rects
/// prevents a transient layout outside the canvas from producing invalid NDC.
pub fn viewport_within_canvas(canvas: RectLike, target: RectLike) -> Option<NormalizedViewport> {
    if canvas.width.is_nan()
        || canvas.width <= 0.0
        || canvas.height.is_nan()
        || canvas.height <= 0.0
    {
        return None;
    }
    let left = clamp(target.left - canvas.left, 0.0, canvas.width);
    let top = clamp(target.top - canvas.top, 0.0, canvas.height);
    let right = clamp(target.left + target.width - canvas.left, 0.0, canvas.width);
    let bottom = clamp(target.top + target.height - canvas.top, 0.0, canvas.height);
    if right - left < 1.0 || bottom - top < 1.0 {
        return None;
    }
    Some(NormalizedViewport {
        left: left / canvas.width,
        top: top / canvas.height,
        right: right / canvas.width,
        bottom: bottom / canvas.height,
    })
}

/// Projects world-space points and returns their NDC bounding box.
pub fn projected_bounds(matrices: &[[f32; 16]], points: &[[f64; 3]]) -> NdcBounds {
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for matrix in matrices {
        for &[x, y, z] in points {
            let clip_x = f64::from(matrix[0]) * x
                + f64::from(matrix[4]) * y
                + f64::from(matrix[8]) * z
                + f64::from(matrix[12]);
            let clip_y = f64::from(matrix[1]) * x
                + f64::from(matrix[5]) * y
                + f64::from(matrix[9]) * z
                + f64::from(matrix[13]);
            let clip_w = f64::from(matrix[3]) * x
                + f64::from(matrix[7]) * y
                + f64::from(matrix[11]) * z
                + f64::from(matrix[15]);
            if !clip_w.is_finite() || clip_w.abs() < 1e-6 {
                continue;
            }
            let ndc_x = clip_x / clip_w;
            let ndc_y = clip_y / clip_w;
            x0 = number_min(x0, ndc_x);
            y0 = number_min(y0, ndc_y);
            x1 = number_max(x1, ndc_x);
            y1 = number_max(y1, ndc_y);
        }
    }
    if ![x0, y0, x1, y1].iter().all(|value| value.is_finite()) {
        NdcBounds {
            x0: -1.0,
            y0: -1.0,
            x1: 1.0,
            y1: 1.0,
        }
    } else {
        NdcBounds { x0, y0, x1, y1 }
    }
}

/// Aligns the centre of the projected silhouette with the DOM slot.
pub fn align_projection(bounds: NdcBounds, viewport: NormalizedViewport) -> ProjectionFraming {
    ProjectionFraming {
        scale: 1.0,
        offset: [
            viewport.left + viewport.right - 1.0 - (bounds.x0 + bounds.x1) / 2.0,
            1.0 - viewport.top - viewport.bottom - (bounds.y0 + bounds.y1) / 2.0,
        ],
    }
}

/// Starts close to the prism and backs the camera away until the projected
/// silhouette fits the viewport. With no artificial padding, either its width
/// or height lands exactly on the slot boundary.
pub fn fit_projection_distance(
    viewport: NormalizedViewport,
    bounds_at_distance: impl Fn(f64) -> NdcBounds,
    min_distance: f64,
    max_distance: f64,
    iterations: usize,
) -> ProjectionDistanceFit {
    let mut near = number_max(1e-4, min_distance);
    let mut far = number_max(near, max_distance);
    let mut bounds = bounds_at_distance(far);
    // Preserve containment even if a future geometry/FOV combination exceeds
    // the initial search range.
    for _ in 0..8 {
        if fits_viewport(bounds, viewport) {
            break;
        }
        far *= 2.0;
        bounds = bounds_at_distance(far);
    }
    for _ in 0..iterations {
        let distance = (near + far) / 2.0;
        let candidate = bounds_at_distance(distance);
        if fits_viewport(candidate, viewport) {
            far = distance;
            bounds = candidate;
        } else {
            near = distance;
        }
    }
    ProjectionDistanceFit {
        distance: far,
        bounds,
        framing: align_projection(bounds, viewport),
    }
}

/// Left-multiplies a view-projection by a scale/translation in clip space.
/// Identity borrows the original, matching upstream's unchanged-buffer result.
pub fn apply_projection_framing(
    view_projection: &[f32; 16],
    framing: ProjectionFraming,
) -> Cow<'_, [f32; 16]> {
    if framing.scale == IDENTITY_PROJECTION_FRAMING.scale
        && framing.offset[0] == 0.0
        && framing.offset[1] == 0.0
    {
        return Cow::Borrowed(view_projection);
    }
    let mut result = *view_projection;
    for column in 0..4 {
        let row = column * 4;
        let w = f64::from(view_projection[row + 3]);
        result[row] =
            (f64::from(view_projection[row]) * framing.scale + w * framing.offset[0]) as f32;
        result[row + 1] =
            (f64::from(view_projection[row + 1]) * framing.scale + w * framing.offset[1]) as f32;
    }
    Cow::Owned(result)
}

/// How much farther a centred wall must extend so the shifted projection still
/// covers every canvas edge. Values never shrink the existing safety margin.
pub fn framing_coverage(framing: ProjectionFraming) -> [f64; 2] {
    let scale = number_max(framing.scale, 1e-4);
    [
        number_max(1.0, (1.0 + framing.offset[0].abs()) / scale),
        number_max(1.0, (1.0 + framing.offset[1].abs()) / scale),
    ]
}

fn fits_viewport(bounds: NdcBounds, viewport: NormalizedViewport) -> bool {
    let available_width = (viewport.right - viewport.left) * 2.0;
    let available_height = (viewport.bottom - viewport.top) * 2.0;
    bounds.x1 - bounds.x0 <= available_width && bounds.y1 - bounds.y0 <= available_height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn viewport_is_relative_clipped_and_nonempty() {
        assert_eq!(
            viewport_within_canvas(
                RectLike {
                    left: 200.0,
                    top: 80.0,
                    width: 1000.0,
                    height: 500.0
                },
                RectLike {
                    left: 700.0,
                    top: 130.0,
                    width: 450.0,
                    height: 400.0
                },
            ),
            Some(NormalizedViewport {
                left: 0.5,
                top: 0.1,
                right: 0.95,
                bottom: 0.9
            })
        );
        let canvas = RectLike {
            left: 100.0,
            top: 50.0,
            width: 400.0,
            height: 200.0,
        };
        assert_eq!(
            viewport_within_canvas(
                canvas,
                RectLike {
                    left: 0.0,
                    top: 0.0,
                    width: 300.0,
                    height: 400.0
                }
            ),
            Some(NormalizedViewport {
                left: 0.0,
                top: 0.0,
                right: 0.5,
                bottom: 1.0
            })
        );
        assert_eq!(
            viewport_within_canvas(
                canvas,
                RectLike {
                    left: 600.0,
                    top: 50.0,
                    width: 100.0,
                    height: 100.0
                }
            ),
            None
        );
        assert_eq!(
            viewport_within_canvas(
                RectLike {
                    width: f64::NAN,
                    ..canvas
                },
                canvas
            ),
            None
        );
        assert_eq!(
            viewport_within_canvas(
                RectLike {
                    height: 0.0,
                    ..canvas
                },
                canvas
            ),
            None
        );
    }

    #[test]
    fn framing_changes_xy_without_changing_depth_or_input() {
        let matrix = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let identity = apply_projection_framing(&matrix, IDENTITY_PROJECTION_FRAMING);
        assert!(matches!(identity, Cow::Borrowed(_)));
        assert!(std::ptr::eq(identity.as_ref(), &matrix));
        let result = apply_projection_framing(
            &matrix,
            ProjectionFraming {
                scale: 0.5,
                offset: [0.25, -0.2],
            },
        );
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result[0], 0.5);
        assert_eq!(result[5], 0.5);
        assert_eq!(result[12], 0.25);
        assert_eq!(result[13], -0.2);
        assert_eq!(result[10], 1.0);
        assert_eq!(result[15], 1.0);
        assert_eq!(matrix[12], 0.0);
    }

    #[test]
    fn camera_search_finds_first_contained_distance() {
        let viewport = NormalizedViewport {
            left: 0.5,
            top: 0.0,
            right: 1.0,
            bottom: 1.0,
        };
        let fit = fit_projection_distance(
            viewport,
            |distance| NdcBounds {
                x0: -1.0 / distance,
                y0: -0.5 / distance,
                x1: 1.0 / distance,
                y1: 0.5 / distance,
            },
            0.5,
            10.0,
            36,
        );
        close(fit.distance, 2.0);
        assert_eq!(fit.framing.scale, 1.0);
        assert_eq!(fit.framing.offset, [0.5, 0.0]);
        let expanded = fit_projection_distance(
            viewport,
            |distance| NdcBounds {
                x0: -100.0 / distance,
                y0: -0.5 / distance,
                x1: 100.0 / distance,
                y1: 0.5 / distance,
            },
            0.5,
            1.0,
            36,
        );
        close(expanded.distance, 200.0);
    }

    #[test]
    fn projection_handles_degenerate_points_and_extends_wall() {
        assert_eq!(
            projected_bounds(&[], &[]),
            NdcBounds {
                x0: -1.0,
                y0: -1.0,
                x1: 1.0,
                y1: 1.0
            }
        );
        let matrix = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        assert_eq!(
            projected_bounds(&[matrix], &[[-0.6, -0.2, 0.0], [0.4, 0.8, 0.0]]),
            NdcBounds {
                x0: -0.6,
                y0: -0.2,
                x1: 0.4,
                y1: 0.8
            }
        );
        let framing = align_projection(
            NdcBounds {
                x0: -0.6,
                y0: -0.2,
                x1: 0.4,
                y1: 0.8,
            },
            NormalizedViewport {
                left: 0.5,
                top: 0.1,
                right: 0.95,
                bottom: 0.9,
            },
        );
        close(framing.offset[0], 0.55);
        close(framing.offset[1], -0.3);
        assert_eq!(framing_coverage(IDENTITY_PROJECTION_FRAMING), [1.0, 1.0]);
        let coverage = framing_coverage(ProjectionFraming {
            scale: 0.8,
            offset: [0.4, -0.1],
        });
        close(coverage[0], 1.75);
        close(coverage[1], 1.375);
    }
}
