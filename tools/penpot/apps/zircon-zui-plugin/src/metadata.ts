export const ZUI_METADATA_NAMESPACE = 'dev.zircon.zui';
export const ZUI_METADATA_BRIDGE_VERSION = 'bridge-version';
export const ZUI_METADATA_ROLE = 'role';
export const ZUI_METADATA_DOCUMENT = 'document';
export const ZUI_METADATA_FILE_NAME = 'file-name';
export const ZUI_METADATA_NODE_ID = 'node-id';
export const ZUI_METADATA_COMPONENT = 'component';
export const ZUI_METADATA_CHILD_INDEX = 'child-index';
export const ZUI_METADATA_BASELINE = 'baseline';
export const ZUI_METADATA_BASELINES = 'baselines';
export const ZUI_METADATA_TEXT_PROPERTY = 'text-property';
export const ZUI_METADATA_EMPTY_TEXT = 'empty-text';
/** Base weight retained when a same-shape rich-text range makes Penpot report `mixed`. */
export const ZUI_METADATA_TEXT_BASE_WEIGHT = 'text-base-weight';
export const ZUI_METADATA_TEXT_STYLE_GUARD = 'text-style-guard';
export const ZUI_METADATA_LAYOUT_SIZING_GUARD = 'layout-sizing-guard';
export const ZUI_METADATA_CONTAINER_KIND = 'container-kind';
/** Projected Penpot layout algorithm, available before the host proxy settles. */
export const ZUI_METADATA_LAYOUT_MODE = 'layout-mode';
/** Axis declared by an authored scroll container for visual overflow audit. */
export const ZUI_METADATA_SCROLL_AXIS = 'scroll-axis';
// Marks an authored Overlay container for projection-only visual auditing.
// This is deliberately distinct from the normalized `free` layout kind.
export const ZUI_METADATA_EXPLICIT_OVERLAY = 'explicit-overlay';
export const ZUI_METADATA_VISUAL_DETACHED = 'visual-detached';
export const ZUI_METADATA_VISUAL_DETACHED_PARENT = 'visual-detached-parent';

export const ZUI_ROLE_ASSET = 'asset';
export const ZUI_ROLE_NODE = 'node';
export const ZUI_ROLE_AUXILIARY = 'auxiliary';
export const ZUI_ROLE_TEXT = 'text';
export const ZUI_ROLE_DETACHED = 'detached';
