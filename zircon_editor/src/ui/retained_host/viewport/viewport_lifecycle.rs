use std::collections::BTreeSet;
use std::sync::Arc;

use crate::scene::viewport::{
    RenderFramework, RenderFrameworkError, RenderViewportDescriptor, RenderViewportHandle,
};
use zircon_runtime_interface::math::UVec2;

use super::active_viewport::ActiveViewport;
use super::editor_viewport_render_defaults::editor_viewport_quality_profile;
use super::retained_viewport_controller::RetainedViewportController;

impl RetainedViewportController {
    pub(super) fn ensure_viewport(
        &self,
        surface_key: &str,
        size: UVec2,
    ) -> Result<Option<(RenderViewportHandle, Arc<dyn RenderFramework>)>, RenderFrameworkError>
    {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        let (render_framework, previous) = {
            let mut shared = self.lock_shared();
            let Some(render_framework) = shared.poll_or_start_render_framework()? else {
                return Ok(None);
            };
            if let Some(viewport) = shared.viewports.get(surface_key).copied() {
                if viewport.size == size {
                    return Ok(Some((viewport.handle, render_framework)));
                }
            }
            (render_framework, shared.viewports.get(surface_key).copied())
        };

        if let Some(viewport) = previous {
            self.clear_viewport_if_current(surface_key, viewport.handle);
            if let Err(error) = render_framework.destroy_viewport(viewport.handle) {
                self.restore_viewport_if_empty(surface_key, viewport);
                return Err(error);
            }
        }

        let label = if surface_key == "editor.viewport" {
            "editor.viewport".to_string()
        } else {
            format!("editor.viewport:{surface_key}")
        };
        let descriptor = RenderViewportDescriptor::new(size).with_label(label);
        let handle = render_framework.create_viewport(descriptor)?;
        if let Err(error) =
            render_framework.set_quality_profile(handle, editor_viewport_quality_profile())
        {
            let _ = render_framework.destroy_viewport(handle);
            return Err(error);
        }

        let mut shared = self.lock_shared();
        shared.viewports.insert(
            surface_key.to_string(),
            ActiveViewport {
                handle,
                size,
                latest_generation: None,
            },
        );
        Ok(Some((handle, render_framework)))
    }

    pub(crate) fn ensure_runtime_viewport(
        &self,
        surface_key: &str,
        size: UVec2,
    ) -> Result<Option<zircon_runtime_interface::ZrRuntimeViewportHandle>, RenderFrameworkError>
    {
        self.ensure_viewport(surface_key, size).map(|viewport| {
            viewport.map(|(handle, _)| {
                zircon_runtime_interface::ZrRuntimeViewportHandle::new(handle.raw())
            })
        })
    }

    fn clear_viewport_if_current(&self, surface_key: &str, expected: RenderViewportHandle) {
        let mut shared = self.lock_shared();
        if shared
            .viewports
            .get(surface_key)
            .is_some_and(|active| active.handle == expected)
        {
            shared.viewports.remove(surface_key);
        }
    }

    fn restore_viewport_if_empty(&self, surface_key: &str, viewport: ActiveViewport) {
        let mut shared = self.lock_shared();
        if !shared.viewports.contains_key(surface_key) {
            shared.viewports.insert(surface_key.to_string(), viewport);
        }
    }

    pub(crate) fn retain_viewport_surfaces(
        &self,
        retained: &BTreeSet<String>,
    ) -> Result<(), RenderFrameworkError> {
        let _operation = self.lock_viewport_lifecycle();
        let (render_framework, retired) = {
            let shared = self.lock_shared();
            let Some(render_framework) = shared.resolve_stored_render_framework()? else {
                return Ok(());
            };
            let retired = shared
                .viewports
                .iter()
                .filter(|(surface, _)| !retained.contains(*surface))
                .map(|(surface, viewport)| (surface.clone(), *viewport))
                .collect::<Vec<_>>();
            (render_framework, retired)
        };
        for (surface, viewport) in retired {
            render_framework.destroy_viewport(viewport.handle)?;
            let mut shared = self.lock_shared();
            if shared
                .viewports
                .get(&surface)
                .is_some_and(|current| current.handle == viewport.handle)
            {
                shared.viewports.remove(&surface);
            }
        }
        Ok(())
    }
}
