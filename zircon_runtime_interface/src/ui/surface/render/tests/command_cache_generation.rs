use serde::ser::SerializeStruct;

use super::{stable_json_generation, FNV_OFFSET};

struct PartialThenFail;

impl serde::Serialize for PartialThenFail {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PartialThenFail", 2)?;
        state.serialize_field("written", &1_u8)?;
        Err(<S::Error as serde::ser::Error>::custom(
            "expected serialization failure",
        ))
    }
}

#[test]
fn ui_render_command_cache_generation_discards_partial_hash_on_serialize_error() {
    assert_eq!(stable_json_generation(&PartialThenFail), FNV_OFFSET);
}

#[test]
fn transient_elements_omit_cache_and_debug_metadata() {
    let command = super::UiRenderCommand {
        node_id: super::UiNodeId::new(1),
        kind: super::UiRenderCommandKind::Group,
        frame: super::UiFrame::new(0.0, 0.0, 32.0, 16.0),
        clip_frame: None,
        z_index: 0,
        style: super::UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: None,
        opacity: 1.0,
    };

    let transient = command.to_transient_paint_elements(0);
    assert!(transient
        .iter()
        .all(|element| element.cache_generation.is_none() && element.debug_label.is_none()));

    let cached = command.to_paint_elements(0);
    assert!(cached
        .iter()
        .all(|element| element.cache_generation.is_some() && element.debug_label.is_some()));
}
