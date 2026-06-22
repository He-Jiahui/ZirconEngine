use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    layout::{compute_virtual_list_window, solve_axis_constraints},
    surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface},
    tree::{
        UiHitTestIndex, UiRuntimeTreeFocusExt, UiRuntimeTreeInteractionExt, UiRuntimeTreeLayoutExt,
        UiRuntimeTreeScrollExt,
    },
};
use zircon_runtime_interface::ui::{
    binding::{UiBindingDirtyDomain, UiBindingSourceKind, UiBindingTargetKind, UiEventKind},
    component::{UiValue, UiValueKind},
    dispatch::{UiNavigationDispatchEffect, UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiReflectedPropertySource, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, Pivot, Position,
        StretchMode, UiAxis, UiContainerKind, UiFrame, UiLayoutEngineBackend, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
        UiVirtualListWindow, UiWrapBoxConfig,
    },
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{
        UiFocusState, UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
        UiRenderCommandKind, UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
        UiVisualAssetRef,
    },
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

mod box_flow;
mod input_visibility;
mod layout_surface;
mod navigation;
mod scroll_mutation;

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn taffy_fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        priority: 0,
        ..fixed_constraint(size)
    }
}
