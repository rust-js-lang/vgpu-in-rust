//! Retained optical/camera state from runtime/state.ts. GPU ownership stays in web.
use crate::camera::{CameraView, camera_view};
use crate::controls::PrismControls;
use crate::framing::{
    IDENTITY_PROJECTION_FRAMING, NormalizedViewport, ProjectionFraming, apply_projection_framing,
    fit_projection_distance, projected_bounds,
};
use crate::light_mesh::{LightMeshLayout, LightMeshOptions};
use crate::normalize_controls::{dispersion_preset, normalize_controls};
use crate::prism_mesh::prism_mesh_data;
use crate::runtime::{lamp_at, light_wall_extent, wall_extent};
use crate::types::{
    CAMERA_DISTANCE, PRISM_BACK_Z, PRISM_DEFAULT_ARC, PRISM_FRONT_Z, PRISM_TRIANGLE,
};
use crate::{clamp, number_max};
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::LazyLock;

static PRISM_FRAME_POINTS: LazyLock<Vec<[f64; 3]>> = LazyLock::new(|| {
    prism_mesh_data(*PRISM_TRIANGLE, PRISM_BACK_Z, PRISM_FRONT_Z)
        .vertices
        .as_chunks::<6>()
        .0
        .iter()
        .map(|vertex| {
            [
                f64::from(vertex[0]),
                f64::from(vertex[1]),
                f64::from(vertex[2]),
            ]
        })
        .collect()
});

pub struct RuntimeState {
    pub output_size: [f64; 2],
    pub controls: PrismControls,
    pub lamp_arc: f64,
    pub lamp_target: f64,
    pub orbit: [f64; 2],
    pub aspect: f64,
    pub camera_distance: f64,
    pub framing_viewport: Option<NormalizedViewport>,
    pub framing: ProjectionFraming,
    pub view: CameraView,
    pub light_mesh_layout: LightMeshLayout,
}
impl RuntimeState {
    pub fn new(output: [f64; 2], layout: LightMeshLayout) -> Self {
        let controls = normalize_controls(&PrismControls::default());
        let aspect = output[0] / number_max(1.0, output[1]);
        let view = camera_view(aspect, 0.0, 0.0, CAMERA_DISTANCE, controls.camera_fov);
        Self {
            output_size: output,
            controls,
            lamp_arc: PRISM_DEFAULT_ARC,
            lamp_target: 0.5,
            orbit: [0.0; 2],
            aspect,
            camera_distance: CAMERA_DISTANCE,
            framing_viewport: None,
            framing: IDENTITY_PROJECTION_FRAMING,
            view,
            light_mesh_layout: layout,
        }
    }
    /// Returns whether GPU spectral vertices need rebuilding. Changes to
    /// presentation-only controls leave camera and mesh identities untouched.
    pub fn set_controls(&mut self, controls: PrismControls) -> bool {
        let next = normalize_controls(&controls);
        let optics_changed = next.dispersion != self.controls.dispersion
            || next.spectral_dispersion != self.controls.spectral_dispersion
            || next.beam_width != self.controls.beam_width
            || next.beam_mouse_y != self.controls.beam_mouse_y
            || next.light_fade.edge_falloff != self.controls.light_fade.edge_falloff;
        let camera_changed = next.camera_fov != self.controls.camera_fov;
        self.controls = next;
        if camera_changed {
            self.refresh_framing();
            self.refresh_camera();
        }
        optics_changed || camera_changed
    }
    pub fn set_lamp_aim(&mut self, arc: f64, target: f64) -> bool {
        let arc = clamp(arc, 0.0, 1.0);
        let target = clamp(target, 0.0, 1.0);
        if arc == self.lamp_arc && target == self.lamp_target {
            return false;
        }
        self.lamp_arc = arc;
        self.lamp_target = target;
        true
    }
    pub fn set_orbit(&mut self, x: f64, y: f64) {
        self.orbit = [clamp(x, -1.0, 1.0), clamp(y, -1.0, 1.0)];
        self.refresh_camera();
    }
    pub fn set_framing_viewport(&mut self, viewport: Option<NormalizedViewport>) -> bool {
        if same_viewport(self.framing_viewport, viewport) {
            return false;
        }
        self.framing_viewport = viewport;
        self.refresh_framing();
        self.refresh_camera();
        true
    }
    pub fn resize(&mut self, output: [f64; 2]) -> bool {
        if self.output_size == output {
            return false;
        }
        self.output_size = output;
        self.aspect = output[0] / number_max(1.0, output[1]);
        self.refresh_framing();
        self.refresh_camera();
        true
    }
    pub fn set_light_mesh_layout(&mut self, layout: LightMeshLayout) -> bool {
        if self.light_mesh_layout.samples == layout.samples
            && self.light_mesh_layout.beam_slices == layout.beam_slices
        {
            return false;
        }
        self.light_mesh_layout = layout;
        true
    }
    pub fn wall_extent(&self) -> [f64; 2] {
        wall_extent(
            self.aspect,
            self.camera_distance,
            self.controls.camera_fov,
            self.framing,
        )
    }
    pub fn light_mesh_options(&self) -> LightMeshOptions {
        LightMeshOptions {
            light: lamp_at(
                self.lamp_arc,
                self.controls.beam_width,
                self.lamp_target,
                self.controls.beam_mouse_y,
            ),
            dispersion: self
                .controls
                .spectral_dispersion
                .unwrap_or_else(|| dispersion_preset(self.controls.dispersion)),
            wall_half_extent: light_wall_extent(
                self.aspect,
                self.camera_distance,
                self.controls.camera_fov,
                self.framing,
            ),
            triangle: None,
            samples: Some(self.light_mesh_layout.samples),
            beam_slices: Some(self.light_mesh_layout.beam_slices),
            exposure: None,
            edge_falloff: Some(self.controls.light_fade.edge_falloff),
        }
    }
    fn refresh_camera(&mut self) {
        let mut view = camera_view(
            self.aspect,
            self.orbit[0],
            self.orbit[1],
            self.camera_distance,
            self.controls.camera_fov,
        );
        if let Cow::Owned(matrix) = apply_projection_framing(&view.view_projection, self.framing) {
            view.view_projection = Rc::new(matrix);
        }
        self.view = view;
    }
    fn refresh_framing(&mut self) {
        match self.framing_viewport {
            None => {
                self.camera_distance = CAMERA_DISTANCE;
                self.framing = IDENTITY_PROJECTION_FRAMING;
            }
            Some(viewport) => {
                let fit = fit_projection_distance(
                    viewport,
                    |distance| {
                        let mut matrices = Vec::new();
                        for orbit_x in [-1.0, 0.0, 1.0] {
                            for orbit_y in [-1.0, 0.0, 1.0] {
                                matrices.push(
                                    *camera_view(
                                        self.aspect,
                                        orbit_x,
                                        orbit_y,
                                        distance,
                                        self.controls.camera_fov,
                                    )
                                    .view_projection,
                                );
                            }
                        }
                        projected_bounds(&matrices, &PRISM_FRAME_POINTS)
                    },
                    PRISM_FRONT_Z + 0.1,
                    32.0,
                    36,
                );
                self.camera_distance = fit.distance;
                self.framing = fit.framing;
            }
        }
    }
}
fn same_viewport(a: Option<NormalizedViewport>, b: Option<NormalizedViewport>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            (a.left - b.left).abs() < 1e-5
                && (a.top - b.top).abs() < 1e-5
                && (a.right - b.right).abs() < 1e-5
                && (a.bottom - b.bottom).abs() < 1e-5
        }
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::light_mesh::light_mesh_layout;
    #[test]
    fn responsive_fit_contains_the_rounded_prism_through_the_whole_orbit() {
        let mut state = RuntimeState::new([1280.0, 720.0], light_mesh_layout(128, 24));
        let viewport = NormalizedViewport {
            left: 0.4,
            top: 0.1,
            right: 0.95,
            bottom: 0.9,
        };
        assert!(state.set_framing_viewport(Some(viewport)));
        let matrix = Rc::clone(&state.view.view_projection);
        assert!(!state.set_framing_viewport(Some(NormalizedViewport {
            left: viewport.left + 1e-6,
            ..viewport
        })));
        assert!(Rc::ptr_eq(&matrix, &state.view.view_projection));
        for x in [-1.0, 0.0, 1.0] {
            for y in [-1.0, 0.0, 1.0] {
                state.set_orbit(x, y);
                let bounds = projected_bounds(&[*state.view.view_projection], &PRISM_FRAME_POINTS);
                assert!(bounds.x0 >= viewport.left * 2.0 - 1.0 - 2e-6);
                assert!(bounds.x1 <= viewport.right * 2.0 - 1.0 + 2e-6);
                assert!(bounds.y0 >= 1.0 - viewport.bottom * 2.0 - 2e-6);
                assert!(bounds.y1 <= 1.0 - viewport.top * 2.0 + 2e-6);
            }
        }
        assert!(state.set_framing_viewport(None));
        assert_eq!(state.camera_distance, CAMERA_DISTANCE);
        assert!(Rc::ptr_eq(
            &state.view.view_projection,
            &state.view.camera.view_projection
        ));
    }
    #[test]
    fn retained_state_only_invalidates_geometry_for_optical_changes() {
        let mut state = RuntimeState::new([1280.0, 720.0], light_mesh_layout(128, 24));
        let matrix = Rc::clone(&state.view.view_projection);
        assert!(!state.resize([1280.0, 720.0]));
        assert!(!state.set_lamp_aim(0.5, 0.5));
        let mut controls = state.controls.clone();
        controls.postprocess.bloom_radius = 2.0;
        assert!(!state.set_controls(controls));
        assert!(Rc::ptr_eq(&matrix, &state.view.view_projection));
        let mut controls = state.controls.clone();
        controls.beam_width = 0.1;
        assert!(state.set_controls(controls));
        assert!(state.set_lamp_aim(-1.0, 2.0));
        assert_eq!([state.lamp_arc, state.lamp_target], [0.0, 1.0]);
        assert!(state.resize([720.0, 1280.0]));
        assert_eq!(state.aspect, 720.0 / 1280.0);
    }
}
