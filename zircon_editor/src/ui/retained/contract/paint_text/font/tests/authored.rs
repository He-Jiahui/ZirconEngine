use super::super::{font_request_for_face, runtime_text_style_for_face, HostTextFontFace};
use super::*;
use zircon_runtime_interface::ui::surface::{UiTextOverflow, UiTextWrap};

fn request(family: &str, weight: u16) -> HostTextFontRequest {
    HostTextFontRequest {
        face: HostTextFontFace::Ui,
        family: family.into(),
        weight,
    }
}

#[test]
fn authored_request_scope_restores_nested_legacy_and_early_return_paths() {
    assert!(active_authored_font_request().is_none());
    let fallback = font_request_for_face(HostTextFontFace::Ui);
    let first = request("Authored outer", 450);
    let second = request("Authored inner", 700);
    {
        let _outer = HostTextFontRequestScope::enter(Some(&first));
        assert_eq!(font_request_for_face(HostTextFontFace::Ui), first);
        {
            let _inner = HostTextFontRequestScope::enter(Some(&second));
            let style = runtime_text_style_for_face(
                HostTextFontFace::Ui,
                17.0,
                8.0,
                UiTextWrap::None,
                UiTextOverflow::Clip,
            );
            assert_eq!(style.font_family.as_deref(), Some("Authored inner"));
            assert_eq!(
                (style.font_weight, style.font_size, style.line_height),
                (700, 17.0, 8.0)
            );
        }
        assert_eq!(active_authored_font_request(), Some(first.clone()));
        {
            let _legacy = HostTextFontRequestScope::enter(None);
            assert_eq!(font_request_for_face(HostTextFontFace::Ui), fallback);
        }
        let early = || {
            let _inner = HostTextFontRequestScope::enter(Some(&second));
            return;
        };
        early();
        assert_eq!(active_authored_font_request(), Some(first.clone()));
    }
    assert!(active_authored_font_request().is_none());
    assert_eq!(font_request_for_face(HostTextFontFace::Ui), fallback);
}

#[test]
fn authored_request_scope_restores_after_unwind_and_rejects_invalid_requests() {
    let outer = request("Outer", 400);
    let _outer = HostTextFontRequestScope::enter(Some(&outer));
    let result = std::panic::catch_unwind(|| {
        let inner = request("Inner", 500);
        let _inner = HostTextFontRequestScope::enter(Some(&inner));
        panic!("exercise request scope unwind");
    });
    assert!(result.is_err());
    assert_eq!(active_authored_font_request(), Some(outer));
    {
        let invalid = request(" ", 1001);
        let _invalid = HostTextFontRequestScope::enter(Some(&invalid));
        assert!(active_authored_font_request().is_none());
    }
}

#[test]
fn authored_node_request_preserves_exact_weight_and_missing_legacy_behavior() {
    let node = TemplatePaneNodeData {
        font_family: "Fira Sans".into(),
        font_weight: 550,
        ..Default::default()
    };
    let resolved = authored_font_request_for_node(&node, UiTextRunPaintStyle::default()).unwrap();
    assert_eq!(resolved.family, "Fira Sans");
    assert_eq!(resolved.weight, 550);
    assert!(
        authored_font_request_for_node(&Default::default(), UiTextRunPaintStyle::default())
            .is_none()
    );
}
