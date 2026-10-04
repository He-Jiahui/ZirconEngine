import type { ZuiDiagnostic, ZuiDocument } from './zui-document';
import type { ZuiPrefabRole } from './zui-prefab-system';
import type { InputControlProjection } from './zui-input-projection';
import type { SliderProjection } from './zui-slider-projection';
import type { SegmentedProjection } from './zui-segmented-projection';
import type { TableProjection } from './zui-table-projection';
import type { DividerProjection } from './zui-divider-projection';
import type { PropertyRowProjection } from './zui-property-row-projection';
import type { FieldProjection } from './zui-field-projection';

export interface ProjectionGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ProjectionPaint {
  fillColor: string | null;
  fillOpacity: number;
  strokeColor: string | null;
  strokeOpacity: number;
  strokeWidth: number;
  borderRadius: number;
  opacity: number;
}

export interface ProjectionTextRun {
  /** UTF-16 start offset in the displayed, marker-free string. */
  start: number;
  /** UTF-16 end offset (exclusive) in the displayed string. */
  end: number;
  /** Requested normal font weight for this inline run. */
  fontWeight: string;
}

export interface ProjectionText {
  characters: string;
  property:
    | 'text'
    | 'value_text'
    | 'value'
    | 'query'
    | 'placeholder'
    | 'title'
    | 'message'
    | 'label'
    | 'label_text'
    | 'group_label'
    | 'options'
    | null;
  color: string | null;
  colorOpacity: number;
  fontSize: number | null;
  fontWeight: string | null;
  fontFamily: string;
  lineHeight: number;
  align: 'left' | 'center' | 'right' | 'justify' | null;
  /** Inline styles applied to this same text shape; never separate blocks. */
  richTextRuns?: ProjectionTextRun[];
}

export interface ProjectionPadding {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface ProjectionContainer {
  kind: 'free' | 'flex' | 'grid';
  direction: 'row' | 'column';
  wrap: boolean;
  gap: number;
  rowGap: number;
  columnGap: number;
  columns: number;
  rows: number;
  padding: ProjectionPadding;
  alignItems: 'start' | 'center' | 'end' | 'stretch';
  justifyContent:
    | 'start'
    | 'center'
    | 'end'
    | 'space-between'
    | 'space-around'
    | 'space-evenly';
  clip: boolean;
}

export interface ProjectionEditableState {
  geometry: ProjectionGeometry;
  paint: ProjectionPaint;
  text: ProjectionText | null;
  textFragments?: Record<string, ProjectionText>;
  slotPadding?: ProjectionPadding;
  container: ProjectionContainer;
}

export interface ProjectedZuiNode extends ProjectionEditableState {
  nodeId: string;
  component: string;
  name: string;
  /** Authored owner tuple passed through the retained host projection. */
  sourcePath?: string;
  sourceNodeId?: string;
  controlId?: string | null;
  /** Compact JSON list of authored instance call sites. */
  instancePath?: string;
  /** Authored runtime properties that own a native painter projection. */
  sourceParts?: ProjectionSourcePart[];
  /**
   * The runtime keeps this node under its authored parent, but the Penpot
   * projection renders it in the explicit detached lane used for native
   * windows. These fields are projection-only and never enter the editable
   * bridge snapshot.
   */
  visualDetached?: boolean;
  visualDetachedParentId?: string;
  prefabRole: ZuiPrefabRole;
  previewHidden: boolean;
  inputControl?: InputControlProjection;
  slider?: SliderProjection;
  segmented?: SegmentedProjection;
  table?: TableProjection;
  divider?: DividerProjection;
  propertyRow?: PropertyRowProjection;
  field?: FieldProjection;
  children: ProjectedZuiNode[];
}

export interface ProjectionSourcePart {
  nodeId: string;
  property: string;
}

export interface ZuiAssetProjection {
  assetId: string;
  displayName: string;
  rootNodes: ProjectedZuiNode[];
  detachedNodes: ProjectedZuiNode[];
  shapes: ProjectedZuiNode[];
  diagnostics: ZuiDiagnostic[];
}

export interface PenpotShapeSnapshot {
  nodeId: string;
  component: string;
  /** Canonical repo-relative source path for the authored node owner. */
  sourcePath?: string;
  /** Exact raw TOML `[nodes.<key>]` in sourcePath. */
  sourceNodeId?: string;
  /** Authored control identifier, separate from the generated Penpot nodeId. */
  controlId?: string | null;
  /** Compact JSON call-site list; `[]` is a known direct-root node. */
  instancePath?: string;
  parentNodeId: string | null;
  childNodeIds: string[];
  /** Authored properties represented by a native painter owner. */
  sourceParts?: ProjectionSourcePart[];
  baseline: ProjectionEditableState;
  current: ProjectionEditableState;
}

export interface PenpotAssetSnapshot {
  assetId: string;
  rootNodeIds: string[];
  detachedNodeIds: string[];
  shapes: PenpotShapeSnapshot[];
}

export interface ReconciledZuiDocument {
  document: ZuiDocument;
  source?: string;
  diagnostics: ZuiDiagnostic[];
  changes: string[];
}
