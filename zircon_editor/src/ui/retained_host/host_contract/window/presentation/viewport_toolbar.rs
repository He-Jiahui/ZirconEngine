//! Readonly capture metadata follows the existing normal-business snapshot owner.
use std::sync::Arc;
use serde_json::{json, Value};
use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiSurfaceFrame};
use super::super::super::globals::HostContractState;

pub(super) fn set_capture(state: &mut HostContractState, snapshot: Option<(Arc<UiSurfaceFrame>, Arc<Value>)>, mount: Option<UiFrame>, scale: f32) {
    let captured = snapshot.and_then(|(frame, value)| physical(&value, mount, scale).map(|value| (frame,Arc::new(value))));
    let cursor = state.presentation_generation().cursor();
    Arc::make_mut(&mut state.host_presentation).normal_viewport_toolbar = captured.map(|(frame,value)|(cursor,frame,value));
}

fn physical(value: &Value, mount: Option<UiFrame>, scale: f32) -> Option<Value> {
    if value["coordinateSpace"] != "logical-pixels" || !scale.is_finite() || scale <= 0.0 { return None; }
    let mut value=value.clone();
    let mount=mount.unwrap_or_default();
    if !mount.x.is_finite() || !mount.y.is_finite() { return None; }
    for node in value["nodes"].as_array_mut()? {
        for name in ["bounds","clip"] {
            let rect=node[name].as_object_mut()?;
            for (axis,offset) in [("x",mount.x),("y",mount.y),("width",0.0),("height",0.0)] {
                let original=rect.get(axis)?.as_f64()?;
                let physical=(original as f32)*scale+offset;
                if !original.is_finite() || !physical.is_finite() { return None; }
                rect.insert(axis.to_owned(),json!(physical));
            }
        }
        if !node["scroll"].is_null() {
            for axis in ["offset","viewportExtent","contentExtent"] {
                let original=node["scroll"][axis].as_f64()?;
                let physical=(original as f32)*scale;
                if !original.is_finite() || !physical.is_finite() { return None; }
                node["scroll"][axis]=json!(physical);
            }
        }
    }
    value["coordinateSpace"]=json!("physical-pixels");
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::PhysicalSize;
    fn payload() -> Value {
        json!({"coordinateSpace":"logical-pixels","nodes":[{"bounds":{"x":2.0,"y":3.0,"width":10.0,"height":12.0},"clip":{"x":2.0,"y":3.0,"width":10.0,"height":12.0},"scroll":{"offset":3.0,"viewportExtent":12.0,"contentExtent":90.0},"state":{"value":0.25,"valueText":"0.25","checked":false,"selected":null},"fontSize":13.333333}]})
    }
    #[test]
    fn readonly_toolbar_capture_preserves_all_cursors_and_old_generations() {
        let mut state=HostContractState::new(PhysicalSize::new(1280,800));
        let old=state.presentation_generation();
        let queue=(state.external_redraw_queued_count,state.external_redraw_drained_count,state.presentation_rebuild_count);
        let logical=Arc::new(payload());
        set_capture(&mut state,Some((Arc::new(UiSurfaceFrame::default()),Arc::clone(&logical))),None,1.5);
        let captured=state.presentation_generation();
        assert_eq!(old.cursor(),captured.cursor());
        assert_eq!(queue,(state.external_redraw_queued_count,state.external_redraw_drained_count,state.presentation_rebuild_count));
        assert!(old.structure().normal_viewport_toolbar.is_none());
        assert_eq!(captured.structure().normal_viewport_toolbar.as_ref().unwrap().2["nodes"][0]["scroll"]["offset"],4.5);
        assert_eq!(logical["nodes"][0]["scroll"]["offset"],3.0);
        set_capture(&mut state,None,None,1.5);
        assert_eq!(old.cursor(),state.presentation_generation().cursor());
        assert!(state.presentation_generation().structure().normal_viewport_toolbar.is_none());
        assert!(captured.structure().normal_viewport_toolbar.is_some());
    }
    #[test]
    fn toolbar_mount_scales_geometry_once_and_preserves_values_nulls_and_typography() {
        let logical=payload();
        let actual=physical(&logical,Some(UiFrame::new(10.0,20.0,100.0,100.0)),1.5).unwrap();
        assert_eq!(actual["nodes"][0]["bounds"]["x"],13.0);
        assert_eq!(actual["nodes"][0]["bounds"]["y"],24.5);
        assert_eq!(actual["nodes"][0]["bounds"]["width"],15.0);
        assert_eq!(actual["nodes"][0]["state"],logical["nodes"][0]["state"]);
        assert_eq!(actual["nodes"][0]["fontSize"],logical["nodes"][0]["fontSize"]);
        assert!(physical(&actual,None,1.5).is_none());
        assert!(physical(&logical,None,0.0).is_none());
    }
}
