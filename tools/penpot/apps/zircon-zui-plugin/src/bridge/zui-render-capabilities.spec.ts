import {
  projectZuiDocument,
  cloneProjectionSnapshot,
  reconcileZuiDocument,
} from './penpot-projection';
import { normalizeZuiDocument, type ZuiDocument } from './zui-document';

describe('native painter capability boundary', () => {
  it.each(['sample-grid', 'timeline-strip', 'weight-heatmap'])(
    'rejects an unmapped %s canvas instead of an empty board',
    (variant) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: {
          root: {
            component: 'Canvas',
            props: {
              component_role: 'canvas',
              component_variant: `custom ${variant}`,
              samples: [1, 2, 3],
            },
          },
        },
      };
      expect(() => projectZuiDocument(source)).toThrow(
        `mapping unavailable for Canvas (${variant})`,
      );
    },
  );

  it.each(['Icon', 'IconButton', 'SvgIcon', 'Image'])(
    'keeps %s labels in source metadata without painting them over the glyph',
    (component) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: {
          root: {
            component,
            props: {
              icon: 'res://icons/add.svg',
              label: 'Add item',
              text: 'Accessible command',
              custom: { retained: true },
            },
            events: [{ id: 'AddItem', event: 'Click', route: 'items.add' }],
          },
        },
      };
      const projection = projectZuiDocument(source);
      expect(projection.shapes[0].text).toBeNull();
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(source));
    },
  );
  it.each([
    'CommandPalette',
    'ConfirmDialog',
    'Dialog',
    'DragOverlay',
    'NotificationCenter',
    'WorkbenchToast',
  ])(
    'maps %s to authored source parts instead of a generic rectangle',
    (component) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: {
          root: {
            component,
            props: {
              open: false,
              popup_open: false,
              title: 'Dialog title',
              message: 'Dialog message',
              text: component === 'DragOverlay' ? 'Drag item' : undefined,
              payload_label: 'Drag item',
              payload_reference: 'asset://item',
              notifications: [],
              empty_text: 'No notifications',
              window_count: 0,
            },
          },
        },
      };
      const projection = projectZuiDocument(source);
      expect(projection.shapes[0].prefabRole).toBe('panel');
      expect(projection.shapes[0].text?.property).toBe(
        component === 'DragOverlay' ? 'text' : 'title',
      );
      if (component === 'NotificationCenter') {
        expect(projection.shapes[0].paint.fillColor).not.toBe('#151719');
      }
      expect(projection.shapes[0].sourceParts?.length).toBeGreaterThan(0);
      expect(
        projection.shapes[0].sourceParts?.every(
          (part) => part.nodeId === 'root',
        ),
      ).toBe(true);
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(source));
    },
  );

  it.each([
    [
      'AgentChat',
      {
        messages: ['user|Review the narrow shell'],
        composer_text: 'Continue the review',
        streaming: true,
        error: false,
      },
      ['messages', 'composer_text', 'streaming', 'error'],
    ],
    [
      'ChatComposer',
      { composer_text: 'Continue the review', streaming: true },
      ['composer_text', 'streaming'],
    ],
  ])(
    'maps %s chat state to authored source parts',
    (component, props, properties) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: { root: { component, props } },
      };

      const projection = projectZuiDocument(source);
      expect(projection.shapes[0].prefabRole).toBe('panel');
      expect(projection.shapes[0].sourceParts).toEqual(
        properties.map((property) => ({ nodeId: 'root', property })),
      );
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(source));
    },
  );

  it.each([
    [
      'AgentPlan',
      {
        text: 'Agent plan · 2/4 complete',
        collection_items: ['done|Map references', 'active|Build surface'],
        value: 0.5,
        min: 0,
        max: 1,
      },
      ['text', 'collection_items', 'value', 'min', 'max'],
    ],
    [
      'ToolCalls',
      {
        text: 'Tool calls · 3',
        collection_items: ['success|read_file|layout.zui'],
        visible_limit: 3,
      },
      ['text', 'collection_items', 'visible_limit'],
    ],
    [
      'AgentApproval',
      {
        text: 'Approval required',
        value_text: 'Write the generated layout',
        label_text: 'Scope: fixtures',
        options: ['Deny', 'Allow write'],
        approval_state: 'pending',
        destructive: true,
      },
      [
        'text',
        'value_text',
        'label_text',
        'options',
        'approval_state',
        'destructive',
      ],
    ],
    [
      'AIUsage',
      {
        text: 'AI usage',
        value_text: '12k used',
        value: 0.39,
        min: 0,
        max: 1,
      },
      ['text', 'value_text', 'value', 'min', 'max'],
    ],
  ])(
    'maps %s workflow state to source-owned painter parts',
    (component, props, properties) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: { root: { component, props } },
      };

      const projection = projectZuiDocument(source);
      expect(projection.shapes[0].prefabRole).toBe('panel');
      expect(projection.shapes[0].text?.property).toBe('text');
      expect(projection.shapes[0].sourceParts).toEqual(
        properties.map((property) => ({ nodeId: 'root', property })),
      );
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(source));
    },
  );

  it.each([
    [
      'DataGrid',
      {
        text: 'Assets',
        options: ['Name', 'Type', 'Status'],
        collection_items: ['selected|Tree.mesh|Mesh|Ready'],
        empty_text: 'No assets match the filter',
      },
      ['text', 'options', 'collection_items', 'empty_text'],
    ],
    [
      'TreeView',
      {
        text: 'Folders',
        collection_items: ['expanded|Assets', 'selected|Meshes'],
        empty_text: 'No folders',
      },
      ['text', 'collection_items', 'empty_text'],
    ],
  ])(
    'maps %s data-surface state to source-owned painter parts',
    (component, props, properties) => {
      const source: ZuiDocument = {
        asset: { kind: 'component', id: 'test', version: 2 },
        components: { Test: { root: 'root' } },
        nodes: { root: { component, props } },
      };

      const projection = projectZuiDocument(source);
      expect(projection.shapes[0].prefabRole).toBe('panel');
      expect(projection.shapes[0].text?.property).toBe('text');
      expect(projection.shapes[0].sourceParts).toEqual(
        properties.map((property) => ({ nodeId: 'root', property })),
      );
      expect(
        normalizeZuiDocument(
          reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(source));
    },
  );

  it('keeps a materialized painter mapped to its authored prefab source', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Snackbar',
          penpot_prefab_source: 'assets/workbench_toast.zui#WorkbenchToast',
          props: { text: 'Saved', action_label: 'Open' },
        },
      },
    };
    const projection = projectZuiDocument(source);
    expect(projection.shapes[0].sourceParts).toEqual([
      { nodeId: 'root', property: 'text' },
      { nodeId: 'root', property: 'action_label' },
    ]);
  });

  it('preserves explicitly hidden product popups for semantic roundtrips', () => {
    const source: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: { component: 'Overlay', children: [{ node: 'popup' }] },
        popup: {
          component: 'NotificationCenter',
          props: { visibility: 'collapsed', notifications: ['unchanged'] },
        },
      },
    };
    const projection = projectZuiDocument(source);
    expect(
      projection.shapes.find((node) => node.nodeId === 'popup')?.previewHidden,
    ).toBe(true);
    expect(
      normalizeZuiDocument(
        reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
          .document,
      ),
    ).toEqual(normalizeZuiDocument(source));
  });
});
