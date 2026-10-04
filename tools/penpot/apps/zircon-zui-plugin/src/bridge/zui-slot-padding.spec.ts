import fixtureSource from './roundtrip-fixture.zui?raw';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { parseZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { projectSlotPadding } from './zui-slot-padding';
import { applySlotPadding, captureSlotPadding } from '../penpot-slot-padding';
import type { Board } from '@penpot/plugin-types';

function fixture() {
  const document = parseZuiDocument(fixtureSource).document;
  const root = document.nodes!['root'];
  const mount = root.children!.find((child) => child.node === 'actions')!;
  mount.slot!['layout']['padding'] = {
    left: '$space.medium',
    top: 4,
    right: 8,
    bottom: 2,
    runtime_hint: 'preserve',
  };
  return { document, mount };
}

describe('native slot padding projection', () => {
  it('does not create a mapping for a slot without authored padding', () => {
    const { document } = fixture();
    const root = document.nodes!['root'];
    const mount = root.children!.find((child) => child.node === 'actions')!;
    delete mount.slot!['layout']['padding'];
    root.component = 'Overlay';
    root.layout!['container'] = { kind: 'Overlay' };
    expect(
      projectZuiDocument(document).shapes.find(
        (node) => node.nodeId === 'actions',
      )!.slotPadding,
    ).toBeUndefined();
  });

  it('keeps outer slot padding separate from child content padding during dependency expansion', () => {
    const { document, mount } = fixture();
    document.imports = {};
    document.nodes!['actions'].layout!['padding'] = { left: 3, right: 5 };
    new LayoutDependencies().embed(document, 'fixture.zui');
    expect(document.nodes!['actions'].layout!['padding']).toEqual({
      left: 3,
      right: 5,
    });
    const node = projectZuiDocument(document).shapes.find(
      (node) => node.nodeId === 'actions',
    )!;
    expect(node.slotPadding).toEqual({ left: 12, top: 4, right: 8, bottom: 2 });
    expect(node.container.padding).toMatchObject({ left: 3, right: 5 });
    expect(mount.slot!['layout']['padding']['runtime_hint']).toBe('preserve');
  });

  it('preserves no-edit source semantics and updates only edited slot sides', () => {
    const { document } = fixture();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    expect(reconcileZuiDocument(document, snapshot).document).toEqual(document);
    const actions = snapshot.shapes.find(
      (shape) => shape.nodeId === 'actions',
    )!;
    actions.current.slotPadding!.right = 24;
    const result = reconcileZuiDocument(document, snapshot);
    const mount = result.document.nodes!['root'].children!.find(
      (child) => child.node === 'actions',
    )!;
    expect(mount.slot!['layout']['padding']).toEqual({
      left: '$space.medium',
      top: 4,
      right: 24,
      bottom: 2,
      runtime_hint: 'preserve',
    });
    expect(result.document.nodes!['actions']).toEqual(
      document.nodes!['actions'],
    );
    expect(result.changes).toEqual([
      'nodes.root.children.actions.slot.layout.padding.right',
    ]);
  });

  it('rejects unresolved tokens and unmapped free-parent padding explicitly', () => {
    const { document, mount } = fixture();
    mount.slot!['layout']['padding']['left'] = '$missing.token';
    expect(() => projectZuiDocument(document)).toThrow(
      'Unresolved slot padding: actions.left',
    );
    mount.slot!['layout']['padding']['left'] = 12;
    document.nodes!['root'].component = 'Overlay';
    document.nodes!['root'].layout!['container'] = { kind: 'Overlay' };
    expect(() => projectZuiDocument(document)).toThrow(
      'Slot padding on a free/overlay parent',
    );
  });

  it('maps repeated expanded settings windows by authored owner identity', () => {
    const { document, mount } = fixture();
    const windowSourcePath =
      'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
    const preferencesSourcePath =
      'zircon_editor/assets/ui/editor/components/workbench/floating/workbench_preferences.zui';
    const template = document.nodes!['actions'];
    const expandedInstances = [
      {
        nodeId: 'compiled_instance_7',
        instancePath: JSON.stringify([
          {
            sourcePath: windowSourcePath,
            sourceNodeId: 'settings_window',
          },
        ]),
      },
      {
        nodeId: 'expanded_prefab_29',
        instancePath: JSON.stringify([
          {
            sourcePath:
              'zircon_editor/assets/ui/editor/windows/editor_main_frame.zui',
            sourceNodeId: 'active_window_host_slot',
          },
          {
            sourcePath: windowSourcePath,
            sourceNodeId: 'settings_window',
          },
        ]),
      },
    ];

    for (const { nodeId, instancePath } of expandedInstances) {
      document.nodes![nodeId] = {
        ...structuredClone(template),
        component: 'Container',
        penpot_review_source_path: preferencesSourcePath,
        penpot_review_source_node_id: 'preferences',
        penpot_review_instance_path: instancePath,
      };
      expect(
        projectSlotPadding(
          document,
          nodeId,
          mount,
          'free',
          document.nodes![nodeId],
        ),
      ).toEqual({
        top: 4,
        right: 8,
        bottom: 2,
        left: 12,
      });
    }

    document.nodes!['unrelated_settings_window'] = {
      ...structuredClone(template),
      component: 'Container',
      penpot_review_source_path: preferencesSourcePath,
      penpot_review_source_node_id: 'preferences',
      penpot_review_instance_path: JSON.stringify([
        {
          sourcePath: 'zircon_editor/assets/ui/other_window.zui',
          sourceNodeId: 'settings_window',
        },
      ]),
    };
    expect(() =>
      projectSlotPadding(
        document,
        'unrelated_settings_window',
        mount,
        'free',
        document.nodes!['unrelated_settings_window'],
      ),
    ).toThrow('Slot padding on a free/overlay parent');
  });

  it('sets independent Penpot margins and captures the same four sides', () => {
    const child: Record<string, unknown> = { marginType: 'simple' };
    const board = { name: 'actions', layoutChild: child } as unknown as Board;
    const padding = { left: 12, top: 4, right: 8, bottom: 2 };
    applySlotPadding(board, padding);
    expect(child).toEqual({
      marginType: 'multiple',
      leftMargin: 12,
      topMargin: 4,
      rightMargin: 8,
      bottomMargin: 2,
    });
    expect(captureSlotPadding(board, padding)).toEqual(padding);
    expect(captureSlotPadding(board, undefined, true)).toEqual(padding);
    expect(() => captureSlotPadding(board, undefined)).toThrow(
      'Unmapped slot margin edit',
    );
  });

  it('uses the side-margin compatibility surface when the host omits marginType', () => {
    const child: Record<string, unknown> = {};
    const board = {
      name: 'legacy-host',
      layoutChild: child,
    } as unknown as Board;
    const padding = { top: 4, bottom: 2, left: 12, right: 8 };
    applySlotPadding(board, padding);
    expect(child).toEqual({
      topMargin: 4,
      rightMargin: 8,
      bottomMargin: 2,
      leftMargin: 12,
    });
    expect(captureSlotPadding(board, padding)).toEqual(padding);
  });
});
