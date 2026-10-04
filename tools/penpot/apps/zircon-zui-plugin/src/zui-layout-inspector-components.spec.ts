import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { prepareComponentReviewHost } from '../tools/zui-layout-component-hosts';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import {
  canonicalSha256,
  validateWorkbenchPresentation,
} from '../tools/zui-layout-workbench-presentation';
import { normalizeZuiDocument, parseZuiDocument } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';

const repo = resolve(
  fileURLToPath(new URL('.', import.meta.url)),
  '../../../../..',
);
const frame = 'zircon_editor/assets/ui/editor/host/editor_main_frame.zui';
const panel =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui';
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

describe('actual Workbench Inspector consumer', () => {
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
    const source = parseZuiDocument(
      await readFile(resolve(repo, frame), 'utf8'),
    ).document;
    const original = normalizeZuiDocument(source);
    const panelSource = parseZuiDocument(
      await readFile(resolve(repo, panel), 'utf8'),
    ).document;
    const prototype = panelSource.nodes!['component_property_slot_04_row']!;
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
      expect(row['penpot_review_source_node_id']).toBeUndefined();
      expect(row.control_id).toMatch(/^WorkbenchComponentPropertyVirtualRow/);
      if (row.props!['read_only'])
        expect(row.props).toMatchObject({
          editable_text: false,
          input_focusable: false,
          input_clickable: false,
        });
    }
    expect(container.repeat).toEqual(
      panelSource.nodes!['inspector_mesh_properties']!.repeat,
    );
    expect(container.layout).toEqual(
      panelSource.nodes!['inspector_mesh_properties']!.layout,
    );
    const originalPrototype = Object.values(document.nodes ?? {}).find(
      (node) => node.control_id === 'WorkbenchComponentPropertySlot04Row',
    )!;
    expect(
      JSON.parse(originalPrototype['penpot_review_instance_path'] as string),
    ).toContainEqual({
      sourcePath: panel,
      sourceNodeId: 'component_property_slot_04_row',
    });
    expect(originalPrototype.events).toEqual(prototype.events);
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
        (node) => node.textFragments?.['property-value']?.characters,
      ),
    ).toEqual(['1.7453293', '0.1', '200', '2', '3']);
    expect(
      paintedRows.every(
        (node) => node.propertyRow !== undefined && node.geometry.height === 28,
      ),
    ).toBe(true);
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
  });
});
