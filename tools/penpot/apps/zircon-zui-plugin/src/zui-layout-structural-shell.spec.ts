import { structuralShellPendingReason } from '../tools/zui-layout-structural-shell';

const node = (
  nodeId: string,
  component: string,
  parentNodeId: string | null,
  overrides: Partial<{
    visible: boolean;
    detached: boolean;
    text: string;
  }> = {},
) => ({
  nodeId,
  component,
  parentNodeId,
  visible: true,
  detached: false,
  text: '',
  ...overrides,
});

describe('structural dynamic-host evidence', () => {
  it('keeps a slot-only dynamic host pending instead of accepting a blank preview', () => {
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [
          node('root', 'VerticalGroup', null),
          node('content', 'Slot', 'root'),
        ],
      }),
    ).toContain('structural Slot');
  });

  it('keeps retired sample content pending without making it visible', () => {
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [
          node('root', 'VerticalGroup', null),
          node('retired', 'Button', 'root', { visible: false }),
        ],
      }),
    ).toContain('retired');
  });

  it('does not mask a product page or a dynamic host with authored content', () => {
    const slotAudit = {
      semanticNodes: [
        node('root', 'VerticalGroup', null),
        node('content', 'Slot', 'root'),
      ],
    };
    expect(structuralShellPendingReason('product-page', slotAudit)).toBeUndefined();
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [
          node('root', 'VerticalGroup', null),
          node('save', 'Button', 'root'),
        ],
      }),
    ).toBeUndefined();
  });

  it('keeps explicit structural host mounts pending without inventing preview content', () => {
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [
          node('root', 'VerticalGroup', null),
          node('mount', 'Space', 'root'),
        ],
      }),
    ).toContain('host mount');
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [
          node('root', 'VerticalGroup', null),
          node('document', 'Container', 'root'),
        ],
      }),
    ).toContain('host mount');
    expect(
      structuralShellPendingReason('dynamic-host', {
        semanticNodes: [node('root', 'HorizontalGroup', null)],
      }),
    ).toContain('host mount');
  });
});
