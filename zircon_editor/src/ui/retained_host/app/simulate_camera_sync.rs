use super::*;

impl RetainedEditorHost {
    pub(super) fn sync_simulate_preview_camera(&mut self) {
        let camera = match self.runtime.simulate_preview_camera() {
            Ok(camera) => camera,
            Err(error) => {
                self.last_simulate_camera = None;
                self.set_status_line(format!("Failed to read the Simulate camera: {error}"));
                return;
            }
        };
        let Some((instance, camera)) = camera else {
            self.last_simulate_camera = None;
            return;
        };
        if self.last_simulate_camera == Some((instance, camera)) {
            zircon_runtime::profile_counter!(
                "editor",
                "play.simulate.camera_unchanged_skipped_count",
                1
            );
            return;
        }
        match self.runtime.route_simulate_preview_camera(camera) {
            Ok(true) => self.last_simulate_camera = Some((instance, camera)),
            Ok(false) => self.last_simulate_camera = None,
            Err(error) => {
                self.last_simulate_camera = None;
                self.set_status_line(error.to_string());
            }
        }
    }
}
