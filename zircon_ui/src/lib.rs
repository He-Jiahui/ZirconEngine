//! Runtime UI module skeleton wired into the core runtime.

mod binding;
mod dispatch;
mod event_ui;
mod layout;
mod module;
mod surface;
mod template;
mod tree;

use zircon_module::{EngineModule, ModuleDescriptor};

pub use binding::{
    UiBindingCall, UiBindingParseError, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
    UiEventRouter,
};
pub use dispatch::{
    UiNavigationDispatchContext, UiNavigationDispatchEffect, UiNavigationDispatchInvocation,
    UiNavigationDispatchResult, UiNavigationDispatcher, UiPointerDispatchContext,
    UiPointerDispatchEffect, UiPointerDispatchInvocation, UiPointerDispatchResult,
    UiPointerDispatcher, UiPointerEvent,
};
pub use event_ui::{
    UiActionDescriptor, UiBindingCodec, UiControlRequest, UiControlResponse, UiEventManager,
    UiInvocationContext, UiInvocationError, UiInvocationRequest, UiInvocationResponse,
    UiInvocationResult, UiInvocationSource, UiNodeDescriptor, UiNodeId, UiNodePath, UiNotification,
    UiParameterDescriptor, UiPropertyDescriptor, UiReflectionDiff, UiReflectionSnapshot, UiRouteId,
    UiStateFlags, UiSubscriptionId, UiTreeId, UiValueType,
};
pub use layout::{
    compute_layout_tree, compute_virtual_list_window, solve_axis_constraints, Anchor,
    AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, Pivot, Position,
    ResolvedAxisConstraint, StretchMode, UiAxis, UiContainerKind, UiFrame, UiLinearBoxConfig,
    UiPoint, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize,
    UiVirtualListConfig, UiVirtualListWindow,
};
pub use module::{module_descriptor, UiConfig, UI_MODULE_NAME};
pub use surface::{
    UiFocusState, UiNavigationEventKind, UiNavigationRoute, UiNavigationState, UiPointerButton,
    UiPointerEventKind, UiPointerRoute, UiRenderCommand, UiRenderCommandKind, UiRenderExtract,
    UiRenderList, UiResolvedStyle, UiSurface, UiVisualAssetRef,
};
pub use template::{
    UiActionRef, UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetImports, UiAssetKind,
    UiAssetLoader, UiAssetRoot, UiBindingRef, UiChildMount, UiCompiledDocument,
    UiComponentDefinition, UiComponentParamSchema, UiComponentTemplate, UiDocumentCompiler,
    UiLegacyTemplateAdapter, UiNamedSlotSchema, UiNodeDefinition, UiNodeDefinitionKind, UiSelector,
    UiSelectorToken, UiSlotTemplate, UiStyleDeclarationBlock, UiStyleResolver, UiStyleRule,
    UiStyleScope, UiStyleSheet, UiTemplateBuildError, UiTemplateDocument, UiTemplateError,
    UiTemplateInstance, UiTemplateLoader, UiTemplateNode, UiTemplateSurfaceBuilder,
    UiTemplateTreeBuilder, UiTemplateValidator,
};
pub use tree::{
    UiDirtyFlags, UiHitTestIndex, UiHitTestResult, UiInputPolicy, UiLayoutCache,
    UiTemplateNodeMetadata, UiTree, UiTreeError, UiTreeNode,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct UiModule;

impl EngineModule for UiModule {
    fn module_name(&self) -> &'static str {
        UI_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Runtime UI widgets and layout"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
