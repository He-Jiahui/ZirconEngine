//! A paint command owns this request only for its synchronous text operation.
//! It is not a font registry: Runtime still resolves the requested family/weight.
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::{HostTextFontRequest, font_face_for_paint_style, font_request_for_face_with_preferences};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_text_preferences;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

thread_local! {
    static AUTHORED_REQUEST: RefCell<Option<HostTextFontRequest>> = const { RefCell::new(None) };
}

pub(in crate::ui::retained_host::host_contract) struct HostTextFontRequestScope {
    previous: Option<HostTextFontRequest>,
    _thread: PhantomData<Rc<()>>,
}

impl HostTextFontRequestScope {
    pub(in crate::ui::retained_host::host_contract) fn enter(request: Option<&HostTextFontRequest>) -> Self {
        let request = request.filter(|request| !request.family.trim().is_empty() && (1..=1000).contains(&request.weight));
        let previous = AUTHORED_REQUEST.with_borrow_mut(|active| std::mem::replace(active, request.cloned()));
        Self { previous, _thread: PhantomData }
    }
}

impl Drop for HostTextFontRequestScope {
    fn drop(&mut self) {
        AUTHORED_REQUEST.with_borrow_mut(|active| *active = self.previous.take());
    }
}

pub(in crate::ui::retained_host::host_contract) fn active_authored_font_request() -> Option<HostTextFontRequest> {
    AUTHORED_REQUEST.with_borrow(Clone::clone)
}

pub(in crate::ui::retained_host::host_contract) fn authored_font_request_for_node(
    node: &TemplatePaneNodeData,
    style: UiTextRunPaintStyle,
) -> Option<HostTextFontRequest> {
    let family = node.font_family.trim();
    let weight = u16::try_from(node.font_weight).ok().filter(|weight| (1..=1000).contains(weight));
    let has_metrics = (node.font_size.is_finite() && node.font_size > 0.0)
        || (node.line_height.is_finite() && node.line_height > 0.0)
        || (node.line_height_ratio.is_finite() && node.line_height_ratio > 0.0);
    if family.is_empty() && weight.is_none() && !has_metrics { return None; }
    let mut request = font_request_for_face_with_preferences(font_face_for_paint_style(style), &current_host_text_preferences());
    if !family.is_empty() { request.family = family.to_owned(); }
    if let Some(weight) = weight { request.weight = weight; }
    Some(request)
}

#[cfg(test)]
#[path = "tests/authored.rs"]
mod tests;
