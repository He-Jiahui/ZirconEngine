import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { describe, expect, it } from 'vitest';
import { prepareComponentReviewHost } from '../tools/zui-layout-component-hosts';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import {
  canonicalSha256,
  validateWorkbenchPresentation,
} from '../tools/zui-layout-workbench-presentation';
import { cloneZuiDocument, normalizeZuiDocument, parseZuiDocument } from './bridge/zui-document';
import { cloneProjectionSnapshot, projectZuiDocument } from './bridge/penpot-projection';
import { fieldGeometry } from './bridge/zui-field-geometry';
import { reconcileReviewSource } from './bridge/zui-review-host';
import { deriveSourceRenderInventory } from '../tools/zui-layout-source-render-inventory';
import { sourceSemanticIdentity } from '../tools/zui-layout-semantic-parity';
import type { LayoutReviewCase } from '../tools/zui-layout-review-contract';

const repo = resolve(
  process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../..'),
);
const frame = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const panel =
  'zircon_editor/assets/ui/editor/unreal/components/details.zui';
// The host palette is theme_tokens; the mounted Unreal sources import Starship.
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
type JsonObject = Record<string, unknown>;

// Unit contract DTOs; genuine Scene producers and JSON are covered by Rust owner tests.
function presentation(rotationDegrees: unknown = ['0', '90', '0']): JsonObject {
  const property = (
    id: string,
    label: string,
    value: string,
    editable = false,
  ) => ({ id, label, value, kind: 'f32', editable });
  const value: JsonObject = {
    schema: 'dev.zircon.editor.workbench-presentation',
    version: 1,
    activeLocale: 'en',
    sourceFingerprint: { sourcePath: frame, sha256: 'c'.repeat(64) },
    viewportSnap: {
      translationEnabled: false,
      rotationEnabled: true,
      scaleEnabled: false,
    },
    stateFingerprint: '',
    layout: {
      active_main_page: 'main',
      main_pages: [
        {
          WorkbenchPage: {
            id: 'main',
            title: 'Workbench',
            activity_window: 'main-window',
          },
        },
      ],
      activity_windows: {
        'main-window': {
          window_id: 'main-window',
          descriptor_id: 'editor.main',
          host_mode: 'EmbeddedMainFrame',
          activity_drawers: {},
          content_workspace: {
            Tabs: { node_id: 'document-tabs', tabs: [], active_tab: null },
          },
          menu_overflow_mode: 'Auto',
          region_overrides: {},
          view_overrides: {},
        },
      },
      floating_windows: [],
    },
    window: { id: 'main-window', title: 'Zircon Editor' },
    pages: {
      activeId: 'main',
      items: [
        { id: 'main', title: 'Workbench', activityWindowId: 'main-window' },
      ],
    },
    documents: { activeId: null, items: [] },
    drawers: [],
    hierarchy: { filterQuery: '', expandedIds: [], selectedIds: [], rows: [] },
    inspector: {
      entityId: '1',
      name: 'Camera',
      parent: '',
      translation: ['21', '2', '14.5'],
      rotationDegrees,
      scale: ['1', '1', '1'],
      renderLayerMask: 0,
      components: [
        {
          id: 'native.CameraComponent',
          title: 'Camera',
          properties: [
            property(
              'native.CameraComponent.fov_y_radians',
              'FOV Y (Radians)',
              '1.7453293',
            ),
            property('native.CameraComponent.z_near', 'Near Clip', '0.1'),
            property('native.CameraComponent.z_far', 'Far Clip', '200'),
          ],
        },
        {
          id: 'plugin.First',
          title: 'First',
          properties: [
            property('plugin.First.intensity', 'Intensity', '2', true),
          ],
        },
        {
          id: 'plugin.Second',
          title: 'Second',
          properties: [property('plugin.Second.intensity', 'Intensity', '3')],
        },
      ],
    },
    status: {
      primary: '',
      secondary: null,
      viewportLabel: '',
      projectPath: '',
    },
  };
  rehash(value);
  return value;
}
function rehash(value: JsonObject): void {
  const { stateFingerprint: ignored, ...state } = value;
  void ignored;
  value['stateFingerprint'] = canonicalSha256(state);
}

describe('actual Unreal Details consumer', () => {
  it('validates optional nullable rotation without filling missing identities', () => {
    for (const rotation of [null, ['0', '90', '0']])
      expect(() =>
        validateWorkbenchPresentation(presentation(rotation)),
      ).not.toThrow();
    const legacy = presentation();
    delete (legacy['inspector'] as JsonObject)['rotationDegrees'];
    rehash(legacy);
    expect(() => validateWorkbenchPresentation(legacy)).not.toThrow();
    for (const rotation of [[], ['0', '90'], ['0', 90, '0'], '90'])
      expect(() =>
        validateWorkbenchPresentation(presentation(rotation)),
      ).toThrow(/rotationDegrees/);
    const missing = presentation();
    delete (missing['inspector'] as JsonObject)['entityId'];
    rehash(missing);
    expect(() => validateWorkbenchPresentation(missing)).toThrow(/entityId/);
  });

  // Loads the full authored repository and projects three complete host states.
  it('expands all fields from the real mounted virtual prototype and preserves source ownership', async () => {
    const dependencies = new LayoutDependencies();
    const authoredFiles = await readdir(
      resolve(repo, 'zircon_editor/assets/ui'),
      { recursive: true, withFileTypes: true },
    );
    await dependencies.load(
      repo,
      authoredFiles
        .filter((entry) => entry.isFile() && entry.name.endsWith('.zui'))
        .map((entry) =>
          relative(repo, resolve(entry.parentPath, entry.name)).replaceAll(
            '\\',
            '/',
          ),
        ),
    );
    const originalSource = await readFile(resolve(repo, frame), 'utf8');
    const source = parseZuiDocument(originalSource).document;
    const original = normalizeZuiDocument(source);
    const panelSource = parseZuiDocument(
      await readFile(resolve(repo, panel), 'utf8'),
    ).document;
    const prototype = panelSource.nodes!['prototype']!;
    const legacy = presentation();
    delete (legacy['inspector'] as JsonObject)['rotationDegrees'];
    rehash(legacy);
    const result = await prepareComponentReviewHost(
      repo,
      frame,
      source,
      dependencies,
      theme,
      legacy,
    );
    expect(result).not.toBeNull();
    expect(result!.consumers.some((consumer) => consumer.sourcePath === panel)).toBe(true);
    const document = result!.projection;
    const container = Object.values(document.nodes ?? {}).find(
      (node) => node.control_id === 'WorkbenchInspectorMeshProperties',
    )!;
    const rows = container.children!.map(
      (mount) => document.nodes![mount.node]!,
    );
    expect(rows).toHaveLength(5);
    expect(
      new Set(rows.map((row) => row.props!['inspector_property_field_id']))
        .size,
    ).toBe(5);
    expect(rows.map((row) => row.props!['value'])).toEqual([
      '1.7453293',
      '0.1',
      '200',
      '2',
      '3',
    ]);
    expect(rows.map((row) => row.props!['text'])).toEqual([
      'Camera / FOV Y (Radians)',
      'Camera / Near Clip',
      'Camera / Far Clip',
      'First / Intensity',
      'Second / Intensity',
    ]);
    expect(rows.map((row) => row.props!['read_only'])).toEqual([
      true,
      true,
      true,
      false,
      true,
    ]);
    for (const row of rows) {
      expect(row.component).toBe('InputField');
      expect(row.layout!['height']).toEqual(prototype.layout!['height']);
      expect(row.layout!['width']).toEqual(prototype.layout!['width']);
      expect(row.events).toEqual(prototype.events);
      expect(row['penpot_review_source_node_id']).toBe('root');
      expect(row['penpot_review_source_path']).toBe(
        'zircon_editor/assets/ui/editor/workbench/data/component_property_row.zui',
      );
      expect(JSON.parse(row['penpot_review_instance_path'] as string).at(-1)).toEqual({
        sourcePath: panel, sourceNodeId: 'prototype',
      });
      expect(row.props!['inspector_property_item_key']).toMatch(/^[1-9][0-9]*$/);
      expect(row.control_id).toMatch(/^WorkbenchComponentPropertyVirtualRow/);
      if (row.props!['read_only'])
        expect(row.props).toMatchObject({
          editable_text: false,
          input_focusable: false,
          input_clickable: false,
        });
    }
    expect(container.repeat).toEqual(
      panelSource.nodes!['properties']!.repeat,
    );
    expect(container.layout).toEqual(
      panelSource.nodes!['properties']!.layout,
    );
    const originalPrototype = Object.values(document.nodes ?? {}).find(
      (node) => node.control_id === 'WorkbenchComponentPropertySlot04Row',
    )!;
    expect(
      JSON.parse(originalPrototype['penpot_review_instance_path'] as string),
    ).toContainEqual({
      sourcePath: panel,
      sourceNodeId: 'prototype',
    });
    expect(originalPrototype.events).toEqual(prototype.events);
    for (const row of rows)
      for (const key of ['penpot_review_source_path', 'penpot_review_source_node_id',
        'penpot_review_instance_path'])
        expect(row[key]).toEqual(originalPrototype[key]);

    // Audit the actual prepared virtual owner and mounted rows, including the
    // real unmounted prototype used as authority. Unrelated host media is outside this contract.
    const containerId = Object.entries(document.nodes!).find(([, node]) => node === container)![0];
    const inspectorHost = { ...document, root: { node: containerId } };
    const reviewCase: LayoutReviewCase = {
      id: 'actual-inspector-repeat-identity', sourcePath: frame, host: 'editor',
      viewport: { width: 1280, height: 800 }, dpi: 1, locale: 'en-US',
      state: 'default', data: { workbenchPresentation: legacy },
    };
    const fingerprints = await Promise.all([panel,
      originalPrototype['penpot_review_source_path'] as string].map(async (sourcePath) => ({
      sourcePath, sha256: createHash('sha256').update(await readFile(resolve(repo, sourcePath))).digest('hex'),
    })));
    const inventory = deriveSourceRenderInventory(inspectorHost, reviewCase, fingerprints);
    expect(inventory.errors).toEqual([]);
    expect(inventory.semanticNodeIdentities).toHaveLength(6);
    const repeatIdentities = rows.map((row) => sourceSemanticIdentity({
      sourcePath: row['penpot_review_source_path'] as string,
      sourceNodeId: row['penpot_review_source_node_id'] as string,
      instancePath: row['penpot_review_instance_path'] as string,
      parentSourcePath: container['penpot_review_source_path'] as string,
      parentSourceNodeId: container['penpot_review_source_node_id'] as string,
      parentInstancePath: container['penpot_review_instance_path'] as string,
      itemKey: row.props!['inspector_property_item_key'] as string,
      propertyFieldId: row.props!['inspector_property_field_id'] as string,
    }));
    expect(new Set(repeatIdentities).size).toBe(5);
    for (const identity of repeatIdentities) {
      expect(JSON.parse(identity)).toHaveLength(7);
      expect(inventory.semanticNodeIdentities).toContain(identity);
    }
    const broken = cloneZuiDocument(inspectorHost);
    const [firstMount, secondMount] = broken.nodes![containerId].children!;
    const first = broken.nodes![firstMount.node], second = broken.nodes![secondMount.node];
    second.props!['inspector_property_item_key'] = first.props!['inspector_property_item_key'];
    second.props!['inspector_property_field_id'] = first.props!['inspector_property_field_id'];
    second['penpot_review_generated_product_row'] = structuredClone(first['penpot_review_generated_product_row']);
    expect(deriveSourceRenderInventory(broken, reviewCase, fingerprints).errors.join(' '))
      .toContain('duplicate prepared source identity');
    const duplicateMount = cloneZuiDocument(inspectorHost);
    duplicateMount.nodes![containerId].children!.push({ ...firstMount });
    expect(deriveSourceRenderInventory(duplicateMount, reviewCase, fingerprints).errors.join(' '))
      .toContain('mounts one node more than once');
    for (const property of ['inspector_property_item_key', 'inspector_property_field_id']) {
      const missing = cloneZuiDocument(inspectorHost);
      delete missing.nodes![firstMount.node].props![property];
      expect(deriveSourceRenderInventory(missing, reviewCase, fingerprints).errors.join(' '))
        .toContain('inconsistent marker/key/field');
    }
    const missingBoth = cloneZuiDocument(inspectorHost);
    delete missingBoth.nodes![firstMount.node].props!['inspector_property_item_key'];
    delete missingBoth.nodes![firstMount.node].props!['inspector_property_field_id'];
    expect(deriveSourceRenderInventory(missingBoth, reviewCase, fingerprints).errors.join(' '))
      .toContain('inconsistent marker/key/field');
    const unprovenOwner = cloneZuiDocument(inspectorHost);
    delete unprovenOwner.nodes![containerId].repeat;
    expect(deriveSourceRenderInventory(unprovenOwner, reviewCase, fingerprints).errors.join(' '))
      .toContain('virtual_rows owner');
    const wrongOwner = cloneZuiDocument(inspectorHost);
    wrongOwner.nodes![containerId]['penpot_review_instance_path'] = JSON.stringify([
      { sourcePath: frame, sourceNodeId: 'different-details-owner' },
    ]);
    expect(deriveSourceRenderInventory(wrongOwner, reviewCase, fingerprints).errors.join(' '))
      .toContain('different source owner');

    // Exercise the generated-row branch during another prefab expansion. The
    // saved root identity is already the callee, so no clone callsite is added.
    const reembedded = cloneZuiDocument(inspectorHost);
    reembedded.nodes![firstMount.node].component = 'WorkbenchComponentPropertyRow';
    dependencies.embed(reembedded, frame, theme);
    for (const key of ['penpot_review_source_path', 'penpot_review_source_node_id',
      'penpot_review_instance_path'])
      expect(reembedded.nodes![firstMount.node][key]).toEqual(originalPrototype[key]);
    const missingOriginal = cloneZuiDocument(inspectorHost);
    delete missingOriginal.nodes![firstMount.node]['penpot_review_source_node_id'];
    expect(() => dependencies.embed(missingOriginal, frame, theme))
      .toThrow('no original authored identity');

    const ownerInput = { ...source, penpot_review_host: document,
      penpot_original_source: originalSource, penpot_original_source_path: frame };
    const exported = reconcileReviewSource(ownerInput, cloneProjectionSnapshot(projectZuiDocument(document)));
    expect(exported.changes).toEqual([]);
    expect(exported.source).toBe(originalSource);
    expect(normalizeZuiDocument(exported.document)).toEqual(original);
    const rotation = Object.values(document.nodes ?? {}).find(
      (node) => node.control_id === 'WorkbenchTransformRotationY',
    )!;
    expect(rotation.props).toMatchObject({
      value: '—',
      read_only: true,
      editable_text: false,
      input_focusable: false,
    });
    const backed = await prepareComponentReviewHost(
      repo,
      frame,
      source,
      dependencies,
      theme,
      presentation(),
    );
    expect(
      Object.values(backed!.projection.nodes ?? {}).find(
        (node) => node.control_id === 'WorkbenchTransformRotationY',
      )!.props!['value'],
    ).toBe('90 deg');
    expect(normalizeZuiDocument(source)).toEqual(original);
    const paintedRows = projectZuiDocument(document).shapes.filter((node) =>
      node.controlId?.startsWith('WorkbenchComponentPropertyVirtualRow'),
    );
    expect(paintedRows).toHaveLength(5);
    expect(
      paintedRows.map(
        (node) => node.text?.characters,
      ),
    ).toEqual(['1.7453293', '0.1', '200', '2', '3']);
    expect(
      paintedRows.every(
        (node) => node.field !== undefined && node.propertyRow === undefined && node.geometry.height === 28,
      ),
    ).toBe(true);
    for (const row of paintedRows) {
      const lane = fieldGeometry(row.field!, 255.2, 28).texts['primary'];
      expect(lane).toBeDefined();
      expect(lane.x).toBe(100);
      expect(lane.y).toBeCloseTo(6, 5);
      expect(lane.width).toBeCloseTo(147.2, 5);
      expect(row.textFragments?.['field-label']?.characters).toBe(document.nodes![row.nodeId].props!['text']);
      expect(lane.height).toBeCloseTo(16, 5);
      expect(row.text).toMatchObject({ fontFamily: 'Roboto', fontSize: 13.333333, lineHeight: 1.2 });
    }
    const controls = projectZuiDocument(document).shapes;
    expect(controls.find((node) => node.controlId === 'WorkbenchInspectorTitle')?.text?.characters).toBe('Camera');
    expect(controls.find((node) => node.controlId === 'UnrealDetailsSelectionHint')?.previewHidden).toBe(true);
    for (const control of ['UnrealDetailsNameArea', 'WorkbenchInspectorFilter', 'RightDrawerContentRoot'])
      expect(controls.find((node) => node.controlId === control)?.previewHidden).toBe(false);
    expect(Object.values(document.nodes!).find((node) => node.control_id === 'WorkbenchInspectorFilter')?.props?.['disabled']).toBe(false);
    expect(controls.find((node) => node.controlId === 'WorkbenchAddComponent')?.previewHidden).toBe(false);
    expect(controls.find((node) => node.controlId === 'WorkbenchStatusErrors')?.previewHidden).toBe(false);
    expect(controls.find((node) => node.controlId === 'WorkbenchSceneTitle')?.text?.characters).toBe('Outliner');
    expect(controls.find((node) => node.controlId === 'UnrealDetailsDockTitle')?.text).toMatchObject({ characters: 'Details', property: null });
    const localizedTitle = Object.values(document.nodes!).find((node) => node.control_id === 'UnrealDetailsDockTitle')!;
    expect(localizedTitle.props!['text']).toEqual({ text_key: 'editor.workbench.details.title', table: 'editor' });
    const noSelection = presentation();
    noSelection['inspector'] = null;
    rehash(noSelection);
    const empty = await prepareComponentReviewHost(repo, frame, source, dependencies, theme, noSelection);
    const emptyControls = projectZuiDocument(empty!.projection).shapes;
    expect(emptyControls.find((node) => node.controlId === 'UnrealDetailsDockTitle')?.previewHidden).toBe(false);
    expect(emptyControls.find((node) => node.controlId === 'UnrealDetailsSelectionHint')?.previewHidden).toBe(false);
    expect(emptyControls.find((node) => node.controlId === 'WorkbenchInspectorTitle')?.text?.characters).toBe('No Selection');
    for (const control of ['UnrealDetailsNameArea', 'WorkbenchInspectorFilter', 'RightDrawerContentRoot', 'WorkbenchInspectorMetadata', 'WorkbenchInspectorTransform', 'WorkbenchAddComponent', 'WorkbenchInspectorMesh'])
      expect(emptyControls.find((node) => node.controlId === control)?.previewHidden).toBe(true);
    expect(Object.values(empty!.projection.nodes!).find((node) => node.control_id === 'WorkbenchInspectorFilter')?.props?.['disabled']).toBe(true);
    const none = await prepareComponentReviewHost(
      repo,
      frame,
      source,
      dependencies,
      theme,
      presentation(null),
    );
    expect(
      Object.values(none!.projection.nodes ?? {}).find(
        (node) => node.control_id === 'WorkbenchTransformRotation',
      )!.props!['visibility'],
    ).toBe('visible');
  }, 15_000);
});
