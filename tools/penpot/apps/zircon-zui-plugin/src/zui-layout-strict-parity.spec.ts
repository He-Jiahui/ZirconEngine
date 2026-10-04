import { existsSync } from 'node:fs';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import {
  missingPenpotCaptureProgramPaths,
  rendererFilePath,
  selectedBrowserExecutablePath,
} from '../tools/zui-layout-evidence';
import {
  penpotCasesComplete,
} from '../tools/zui-layout-penpot-cases';
import {
  canonicalSha256,
  caseSha256,
  type LayoutRenderEvidence,
  type LayoutReviewCase,
  validateLayoutReviewCase,
} from '../tools/zui-layout-review-contract';
import {
  compareSemanticGeometry,
  type RendererRuntimeAssets,
} from '../tools/zui-layout-semantic-parity';
import { compareTextEvidence } from '../tools/zui-layout-text-parity';
import {
  capturePenpotRuntimeReceipts,
  penpotStyleReceiptIsRequired,
} from '../tools/zui-layout-penpot-receipts';
import { validateWorkbenchPresentation } from '../tools/zui-layout-workbench-presentation';
import { deriveSourceRenderInventory } from '../tools/zui-layout-source-render-inventory';

const workbenchSourcePath =
  'zircon_editor/assets/ui/editor/components/workbench/modules/workbench_window.zui';
type WorkbenchPresentationFixture = Record<string, unknown> & {
  activeLocale: 'en' | 'zh-CN';
  pages: { items: unknown[] };
  layout: Record<string, unknown>;
};
const workbenchPresentation = (
  activeLocale: 'en' | 'zh-CN' = 'en',
): WorkbenchPresentationFixture => {
  const value: Record<string, unknown> = {
    schema: 'dev.zircon.editor.workbench-presentation',
    version: 1,
    activeLocale,
    sourceFingerprint: {
      sourcePath: workbenchSourcePath,
      sha256: 'c'.repeat(64),
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
      items: [{ id: 'main', title: 'Workbench', activityWindowId: 'main-window' }],
    },
    documents: { activeId: null, items: [] },
    drawers: [],
    hierarchy: { filterQuery: '', expandedIds: [], selectedIds: [], rows: [] },
    inspector: {
      entityId: null,
      name: '',
      parent: '',
      translation: ['0', '0', '0'],
      scale: ['1', '1', '1'],
      renderLayerMask: 0,
      components: [],
    },
    status: { primary: '', secondary: null, viewportLabel: '', projectPath: '' },
  };
  const state = { ...value };
  delete state['stateFingerprint'];
  value['stateFingerprint'] = canonicalSha256(state);
  return value as WorkbenchPresentationFixture;
};
const managedSceneFingerprintFixture = () => ({
  projectRoot: 'D:\\cargo-targets\\workbench-test-project',
  sources: [
    { sourcePath: 'zircon-project.toml', sha256: '0'.repeat(64) },
    { sourcePath: 'assets/scenes/main.scene.toml', sha256: '1'.repeat(64) },
    {
      sourcePath: 'assets/scenes/workbench-review-empty.scene.toml',
      sha256: '2'.repeat(64),
    },
  ],
});
const workbenchCase: LayoutReviewCase = {
  id: 'default-1280x800-dpi1',
  sourcePath: workbenchSourcePath,
  host: 'editor',
  viewport: { width: 1280, height: 800 },
  dpi: 1,
  locale: 'en-US',
  state: 'default',
  data: {
    managedSceneFingerprint: managedSceneFingerprintFixture(),
    workbenchState: {
      sourcePath: workbenchSourcePath,
      controlId: 'WorkbenchWindowRoot',
      sourceNodeId: 'root',
    },
    workbenchPresentation: workbenchPresentation(),
  },
};

const importedWorkbenchSourcePath =
  'zircon_editor/assets/ui/editor/components/workbench/primitives/workbench_button.zui';
const iconPath = 'zircon_editor/assets/icons/grid.svg';
const sourceMapPath = 'tools/editor-host-sources.json';
const sourceMapHash = 'd'.repeat(64);
const sourceFingerprints: Array<[string, string]> = [
  [workbenchSourcePath, 'c'.repeat(64)],
  [importedWorkbenchSourcePath, 'e'.repeat(64)],
];

const compareWorkbenchGeometry = (
  penpot: unknown,
  engine: unknown,
  reviewCase = workbenchCase,
  runtimeAssets: RendererRuntimeAssets = {},
) =>
  compareSemanticGeometry(penpot, engine, reviewCase, {
    ...runtimeAssets,
    expectedSemanticNodes:
      runtimeAssets.expectedSemanticNodes ?? [
        JSON.stringify([workbenchSourcePath, 'root', '[]']),
      ],
    expectedResourceUses: runtimeAssets.expectedResourceUses ?? [],
    expectedHostOverlayIds: runtimeAssets.expectedHostOverlayIds ?? [],
    authoredSources: sourceFingerprints,
    engineCapturePrograms: [[sourceMapPath, sourceMapHash]],
  });

type StyleInventoryFixture = {
  version: 1;
  complete: boolean;
  properties: {
    foregroundColor: string | null;
    backgroundColor: string | null;
    borderColor: string | null;
    borderWidth: number | null;
    borderRadius: number | null;
    opacity: number | null;
    boxShadow: { complete: boolean; layers: Array<Record<string, unknown>> };
  };
};
type SemanticNodeFixture = {
  sourcePath: string;
  nodeId: string;
  sourceNodeId: string;
  instancePath: string;
  controlId: string | null;
  parentNodeId: string | null;
  parentSourcePath: string | null;
  parentSourceNodeId: string | null;
  parentInstancePath: string | null;
  component: string;
  visible: boolean;
  detached: boolean;
  clip: boolean;
  clipBounds: { x: number; y: number; width: number; height: number } | null;
  text: string | null;
  bounds: { x: number; y: number; width: number; height: number };
  styleInventory: StyleInventoryFixture;
  shapeId?: string;
};
type GeometryFixture = {
  case: LayoutReviewCase;
  caseSha256: string;
  coordinateSpace: string;
  windowMetrics: {
    logicalSize: { width: number; height: number };
    physicalSize: { width: number; height: number };
  };
  sourceIdentityProvenance: {
    sourceMapFingerprint: [string, string];
    sources: Array<[string, string]>;
  };
  semanticAudit: { complete: boolean; expectedNodeCount: number };
  assetAudit: {
    complete: boolean;
    resources: Array<{
      kind: 'raster' | 'vector';
      path: string;
      sha256: string;
      loaded: true;
      sourcePath: string;
      sourceNodeId: string;
      instancePath: string;
    }>;
  };
  hostOverlayAudit: { complete: boolean; windows: Array<{ windowId: string }> };
  layout: { semanticNodes: SemanticNodeFixture[] };
};
const styleInventory = (): StyleInventoryFixture => ({
  version: 1,
  complete: true,
  properties: {
    foregroundColor: 'rgba(255,255,255,1)',
    backgroundColor: 'rgba(0,0,0,0)',
    borderColor: 'rgba(0,0,0,0)',
    borderWidth: 0,
    borderRadius: 0,
    opacity: 1,
    boxShadow: { complete: true, layers: [] },
  },
});

const workbenchGeometry = (reviewCase = workbenchCase): GeometryFixture => ({
  case: reviewCase,
  caseSha256: caseSha256(reviewCase),
  coordinateSpace: 'logical',
  windowMetrics: {
    logicalSize: reviewCase.viewport,
    physicalSize: {
      width: reviewCase.viewport.width * reviewCase.dpi,
      height: reviewCase.viewport.height * reviewCase.dpi,
    },
  },
  sourceIdentityProvenance: {
    sourceMapFingerprint: [sourceMapPath, sourceMapHash],
    sources: [[workbenchSourcePath, 'c'.repeat(64)]],
  },
  semanticAudit: { complete: true, expectedNodeCount: 1 },
  assetAudit: { complete: true, resources: [] },
  hostOverlayAudit: { complete: true, windows: [] },
  layout: {
    semanticNodes: [
      {
        sourcePath: workbenchSourcePath,
        nodeId: 'render-root',
        sourceNodeId: 'root',
        instancePath: '[]',
        controlId: 'WorkbenchWindowRoot',
        parentNodeId: null,
        parentSourcePath: null,
        parentSourceNodeId: null,
        parentInstancePath: null,
        component: 'WorkbenchWindow',
        visible: true,
        detached: false,
        clip: false,
        clipBounds: null,
        text: null,
        bounds: { x: 0, y: 0, width: 1280, height: 800 },
        styleInventory: styleInventory(),
      },
    ],
  },
});

const workbenchTextEvidence = () => {
  const geometry = workbenchGeometry();
  geometry.layout.semanticNodes[0].text = 'Hello';
  geometry.layout.semanticNodes[0].shapeId = 'shape-label';
  const fontHash = 'a'.repeat(64);
  const font = {
    familyName: 'Fira Sans',
    postScriptName: 'FiraSans-Regular',
    glyphCount: 5,
    resourcePath: 'resources/fonts/fira.ttf',
    sha256: fontHash,
    sourceUrl: 'https://fonts.example.test/fira.ttf',
  };
  const lineFrame = { x: 10, y: 20, width: 40, height: 16 };
  const penpot = {
    case: workbenchCase,
    caseSha256: caseSha256(workbenchCase),
    coordinateSpace: 'logical',
    fontAudit: { loaded: true },
    texts: [
      {
        shapeId: 'shape-label-text',
        ancestorShapeIds: ['shape-label'],
        text: 'Hello',
        fontFamily: 'Fira Sans, sans-serif',
        fontSize: '14px',
        fontWeight: '400',
        lineHeight: '16px',
        letterSpacing: '0px',
        lineTexts: ['Hello'],
        lines: [lineFrame],
        fonts: [font],
      },
    ],
  };
  const native = {
    case: workbenchCase,
    caseSha256: caseSha256(workbenchCase),
    coordinateSpace: 'logical',
    fontAudit: { loaded: true },
    nodes: [
      {
        sourcePath: workbenchSourcePath,
        nodeId: 'native-text-render-id',
        sourceNodeId: 'root',
        instancePath: '[]',
        controlId: 'WorkbenchWindowRoot',
        text: 'Hello',
        fonts: [font],
        layout: {
          font_size: 14,
          font_family: 'Fira Sans',
          font_weight: 400,
          line_height: 16,
          letter_spacing: 0,
          lines: [{ text: 'Hello', frame: lineFrame }],
        },
      },
    ],
  };
  const runtimeAssets = {
    penpot: [['resources/fonts/fira.ttf', fontHash]] as Array<[string, string]>,
    engine: [['resources/fonts/fira.ttf', fontHash]] as Array<[string, string]>,
  };
  return { penpot, native, geometry, runtimeAssets };
};

const captureMountedStyle = async (
  styleOverrides: Record<string, string>,
  tagName = 'div',
) => {
  const style = {
    display: 'block',
    visibility: 'visible',
    fill: 'none',
    stroke: 'none',
    strokeWidth: '0px',
    opacity: '1',
    boxShadow: 'none',
    filter: 'none',
    color: 'rgb(0,0,0)',
    borderTopLeftRadius: '0px',
    borderTopRightRadius: '0px',
    borderBottomRightRadius: '0px',
    borderBottomLeftRadius: '0px',
    ...styleOverrides,
    getPropertyValue(name: string) {
      if (name === 'fill-opacity') return styleOverrides['fill-opacity'] ?? '1';
      if (name === 'stroke-opacity') return styleOverrides['stroke-opacity'] ?? '1';
      return '';
    },
  };
  const candidate = {
    id: 'shape-receipt',
    tagName,
    parentElement: undefined as unknown,
    querySelectorAll: () => [],
    closest: () => null,
    getAttribute: (name: string) =>
      name === 'rx' || name === 'ry' ? '0' : null,
  };
  const root = {
    id: 'shape-board',
    contains: (element: unknown) => element === root || element === candidate,
    querySelectorAll: () => [],
  };
  candidate.parentElement = root;
  const previous = new Map<string, PropertyDescriptor | undefined>(
    ['document', 'CSS', 'getComputedStyle'].map((key) => [
      key,
      Object.getOwnPropertyDescriptor(globalThis, key),
    ]),
  );
  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: {
      querySelector: () => root,
      getElementById: (id: string) =>
        id === candidate.id ? candidate : undefined,
    },
  });
  Object.defineProperty(globalThis, 'CSS', {
    configurable: true,
    value: { escape: (value: string) => value },
  });
  Object.defineProperty(globalThis, 'getComputedStyle', {
    configurable: true,
    value: () => style,
  });
  try {
    const page = {
      evaluate: async (callback: (value: unknown) => unknown, value: unknown) =>
        await callback(value),
    } as unknown as Parameters<typeof capturePenpotRuntimeReceipts>[0];
    return await capturePenpotRuntimeReceipts(
      page,
      'board',
      [
        {
          shapeId: 'receipt',
          sourcePath: workbenchSourcePath,
          sourceNodeId: 'paint',
          instancePath: '[]',
          visible: true,
          bounds: { x: 0, y: 0, width: 10, height: 10 },
        },
      ],
      [],
      { width: 1280, height: 800 },
    );
  } finally {
    for (const [key, descriptor] of previous) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else Reflect.deleteProperty(globalThis, key);
    }
  }
};

describe('renderer artifact storage roots', () => {
  it.each(['D', 'E', 'F'])(
    'accepts the exact %s drive cargo-targets root',
    async (drive, context) => {
      const artifactBase = `${drive}:\\cargo-targets\\zircon-local`;
      if (!existsSync(artifactBase)) {
        context.skip();
        return;
      }
      const root = await mkdtemp(`${artifactBase}\\zui-layout-test-`);
      const path = resolve(root, 'renderer.bin');
      try {
        await writeFile(path, 'renderer');
        expect(rendererFilePath('E:\\Git\\ZirconEngine', path)).toBe(path);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    },
  );

  it.each([
    'D:\\targets\\zui\\renderer.bin',
    'E:\\ZirconBuilds\\zui\\renderer.bin',
    'F:\\workspace\\cargo-targets\\zui\\renderer.bin',
    'D:\\cargo-targets-old\\zui\\renderer.bin',
    'C:\\cargo-targets\\zui\\renderer.bin',
    'D:\\cargo-targets\\..\\cargo-targets\\zui\\renderer.bin',
    'E:\\Git\\ZirconEngine\\D\\cargo-targets\\renderer.bin',
  ])('rejects an unapproved renderer path: %s', (path) => {
    expect(() => rendererFilePath('E:\\Git\\ZirconEngine', path)).toThrow(
      'Renderer path is outside repository and approved build roots',
    );
  });
});

describe('main-workbench strict semantic parity', () => {
  it('keeps a passed Penpot case incomplete when its resolved browser receipt is malformed', () => {
    const program = [['capture.ts', 'a'.repeat(64)] as [string, string]];
    const rendererSha256 = 'b'.repeat(64);
    const entry: Parameters<typeof penpotCasesComplete>[0] = {
      category: 'workbench',
      name: 'main',
      cases: [workbenchCase],
      sourceSha256: 'c'.repeat(64),
      dependencySha256: 'd'.repeat(64),
      penpotInputSha256: 'e'.repeat(64),
      penpotEvidence: [
        {
          caseId: workbenchCase.id,
          status: 'passed',
          screenshotPath: 'workbench/main/penpot.png',
          screenshotSha256: 'f'.repeat(64),
          sourceSha256: 'c'.repeat(64),
          dependencySha256: 'd'.repeat(64),
          inputSha256: 'e'.repeat(64),
          caseSha256: caseSha256(workbenchCase),
          rendererSha256,
          captureProgramFingerprints: program,
          browserRuntime: {} as NonNullable<LayoutRenderEvidence['browserRuntime']>,
        },
      ],
    };

    expect(penpotCasesComplete(entry, rendererSha256, program)).toBe(false);

    entry.penpotEvidence![0]!.browserRuntime = {
      product: 'Chromium/120.0',
      userAgent: 'Mozilla/5.0 Chrome/120.0',
      executablePath: selectedBrowserExecutablePath(),
      executableSha256: 'a'.repeat(64),
    };
    expect(penpotCasesComplete(entry, rendererSha256, program)).toBe(true);
  });

  it('requires fingerprints for the strict capture and parity toolchain', () => {
    const requiredPaths = [
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-visual-validation.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-visual-source-scope.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-capture-provenance.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-penpot-cases.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-cases.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-managed-inputs.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-artifact-path.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-presentation.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-projection.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/penpot-workbench-canvas-edit.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/penpot-preview-layout-audit.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-evidence.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-penpot-receipts.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-source-render-inventory.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-browser-runtime.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-runtime-provenance.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-semantic-parity.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-text-parity.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-text-evidence.ts',
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-font-resources.ts',
    ];
    const fingerprints = requiredPaths.map(
      (path) => [path, 'a'.repeat(64)] as [string, string],
    );
    expect(missingPenpotCaptureProgramPaths(fingerprints)).toEqual([]);
    expect(
      missingPenpotCaptureProgramPaths(
        fingerprints.filter(
          ([path]) =>
            path !==
            'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-source-render-inventory.ts',
        ),
      ),
    ).toEqual([
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-source-render-inventory.ts',
    ]);
    expect(
      missingPenpotCaptureProgramPaths(
        fingerprints.filter(
          ([path]) =>
            path !==
            'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-evidence.ts',
        ),
      ),
    ).toEqual([
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-evidence.ts',
    ]);
  });

  it('validates bounded product presentation and canonical state fingerprint', () => {
    const presentation = workbenchPresentation();
    expect(() => validateWorkbenchPresentation(presentation)).not.toThrow();

    const stale = {
      ...presentation,
      pages: {
        activeId: 'missing',
        items: presentation['pages'].items,
      },
    };
    expect(() => validateWorkbenchPresentation(stale)).toThrow(
      'stateFingerprint is stale',
    );

    const unsupported = workbenchPresentation();
    unsupported['unknown'] = true;
    const copy = { ...unsupported };
    delete copy['stateFingerprint'];
    unsupported['stateFingerprint'] = canonicalSha256(copy);
    expect(() => validateWorkbenchPresentation(unsupported)).toThrow(
      'Unknown workbenchPresentation field',
    );
  });

  it('validates the hashed workbench-state source selector', () => {
    expect(() => validateLayoutReviewCase(workbenchCase)).not.toThrow();
    expect(() =>
      validateLayoutReviewCase({
        ...workbenchCase,
        data: {
          managedSceneFingerprint: managedSceneFingerprintFixture(),
          workbenchState: { sourcePath: workbenchSourcePath },
          workbenchPresentation: workbenchPresentation(),
        },
      }),
    ).toThrow('workbenchState.controlId');

    const localizedCase: LayoutReviewCase = {
      ...workbenchCase,
      locale: 'zh-CN',
      data: {
        workbenchPresentation: workbenchPresentation('zh-CN'),
        managedSceneFingerprint: managedSceneFingerprintFixture(),
        workbenchState: {
          sourcePath: workbenchSourcePath,
          controlId: 'WorkbenchWindowRoot',
          sourceNodeId: 'root',
          textOverrides: [
            {
              sourcePath: workbenchSourcePath,
              sourceNodeId: 'root',
              controlId: 'WorkbenchWindowRoot',
              property: 'text',
              value: '\u754c\u9762\u6587\u672c',
            },
          ],
        },
      },
    };
    expect(() => validateLayoutReviewCase(localizedCase)).not.toThrow();
    expect(() =>
      validateLayoutReviewCase({
        ...localizedCase,
        data: {
          workbenchPresentation: workbenchPresentation('zh-CN'),
          managedSceneFingerprint: managedSceneFingerprintFixture(),
          workbenchState: {
            sourcePath: workbenchSourcePath,
            controlId: 'WorkbenchWindowRoot',
            sourceNodeId: 'root',
            textOverrides: [
              {
                sourcePath: workbenchSourcePath,
                sourceNodeId: 'root',
                controlId: 'WorkbenchWindowRoot',
                property: 'text',
                value: 'bad\u0000value',
              },
            ],
          },
        },
      }),
    ).toThrow('control character');
    expect(() =>
      validateLayoutReviewCase({
        ...localizedCase,
        data: {
          workbenchPresentation: workbenchPresentation('zh-CN'),
          managedSceneFingerprint: managedSceneFingerprintFixture(),
          workbenchState: {
            sourcePath: workbenchSourcePath,
            controlId: 'WorkbenchWindowRoot',
            sourceNodeId: 'root',
            textOverrides: [
              {
                sourcePath: workbenchSourcePath,
                sourceNodeId: 'root',
                controlId: 'WorkbenchWindowRoot',
                property: 'text',
                value: '\u5b57'.repeat(1366),
              },
            ],
          },
        },
      }),
    ).toThrow('4096 UTF-8 bytes');
  });

  it('requires logical, complete, source-identified semantic geometry', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    expect(
      compareWorkbenchGeometry(penpot, engine).errors,
    ).toEqual([]);

    const physical = workbenchGeometry();
    physical.coordinateSpace = 'physical';
    expect(
      compareWorkbenchGeometry(penpot, physical).errors.join(' '),
    ).toContain('logical');

    const incomplete = workbenchGeometry();
    incomplete.semanticAudit.complete = false;
    expect(
      compareWorkbenchGeometry(penpot, incomplete).errors.join(' '),
    ).toContain('semantic coverage');
  });

  it('keeps offscreen semantic geometry strict while allowing absent paint receipts', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    for (const geometry of [penpot, engine]) {
      geometry.layout.semanticNodes[0].bounds.x = 1281;
      geometry.layout.semanticNodes[0].styleInventory = {
        version: 1,
        complete: false,
        properties: {} as StyleInventoryFixture['properties'],
      };
    }
    expect(compareWorkbenchGeometry(penpot, engine).errors).toEqual([]);

    for (const geometry of [penpot, engine]) {
      geometry.layout.semanticNodes[0].bounds = {
        x: 1260,
        y: 0,
        width: 40,
        height: 10,
      };
      geometry.layout.semanticNodes[0].clip = true;
      geometry.layout.semanticNodes[0].clipBounds = {
        x: 1280,
        y: 0,
        width: 0,
        height: 10,
      };
    }
    expect(compareWorkbenchGeometry(penpot, engine).errors).toEqual([]);

    engine.layout.semanticNodes[0].bounds.y += 2;
    expect(compareWorkbenchGeometry(penpot, engine).errors.join(' ')).toContain(
      'Geometry differs by 2px',
    );
  });

  it('keeps missing Penpot style receipts pending only for paint-visible owners', () => {
    const node = {
      shapeId: 'offscreen-shape',
      visible: true,
      bounds: { x: 1281, y: 0, width: 10, height: 10 },
      clip: false,
      clipBounds: null,
    };
    expect(
      penpotStyleReceiptIsRequired('offscreen-shape', [node], workbenchCase.viewport),
    ).toBe(false);

    node.bounds.x = 0;
    expect(
      penpotStyleReceiptIsRequired('offscreen-shape', [node], workbenchCase.viewport),
    ).toBe(true);
    expect(
      penpotStyleReceiptIsRequired('unknown-shape', [node], workbenchCase.viewport),
    ).toBe(true);
  });

  it('includes SVG fill and stroke opacity in computed receipt colors', async () => {
    const receipts = await captureMountedStyle(
      {
        fill: 'rgb(10,20,30)',
        'fill-opacity': '0.5',
        stroke: 'rgb(40,50,60)',
        'stroke-opacity': '0.25',
        strokeWidth: '2px',
      },
      'rect',
    );
    const style = receipts.styles.get('receipt');

    expect(style?.complete).toBe(true);
    expect(style?.properties.backgroundColor).toBe('rgba(10,20,30,0.5)');
    expect(style?.properties.borderColor).toBe('rgba(40,50,60,0.25)');
  });

  it('keeps asymmetric no-shadow corner radii incomplete', async () => {
    const receipts = await captureMountedStyle({
      borderTopLeftRadius: '4px',
      borderTopRightRadius: '0px',
    });

    expect(receipts.styles.get('receipt')?.complete).toBe(false);
    expect(receipts.pendingReasons.join(' ')).toContain('style receipt');
  });

  it('requires canonical authored instance paths and exact clipping geometry', () => {
    const malformed = workbenchGeometry();
    malformed.layout.semanticNodes[0].instancePath = '[ { } ]';
    expect(compareWorkbenchGeometry(malformed, workbenchGeometry()).errors.join(' ')).toContain(
      'invalid or duplicate semantic node',
    );

    const clipDiffers = workbenchGeometry();
    clipDiffers.layout.semanticNodes[0].clip = true;
    clipDiffers.layout.semanticNodes[0].clipBounds = {
      x: 1,
      y: 0,
      width: 1280,
      height: 800,
    };
    expect(compareWorkbenchGeometry(workbenchGeometry(), clipDiffers).errors.join(' ')).toContain(
      'Semantic ownership differs',
    );
  });

  it('requires a factual empty floating-window audit and fails closed for visible windows', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    expect(compareWorkbenchGeometry(penpot, engine).errors).toEqual([]);

    const missingAudit = { ...penpot, hostOverlayAudit: undefined };
    expect(compareWorkbenchGeometry(missingAudit, engine).errors).toContain(
      'penpot: incomplete host overlay audit',
    );

    const floatingCase: LayoutReviewCase = {
      ...workbenchCase,
      id: 'floating-window',
      data: {
        ...workbenchCase.data,
        workbenchPresentation: {
          ...workbenchPresentation(),
          layout: {
            ...(workbenchPresentation() as { layout: Record<string, unknown> })
              .layout,
            floating_windows: [
              {
                window_id: 'floating-panel',
                title: 'Inspector',
                workspace: { Tabs: { node_id: 'floating-tabs', tabs: [], active_tab: null } },
                focused_view: null,
                frame: { x: 10, y: 20, width: 300, height: 200 },
              },
            ],
          },
        },
      },
    };
    const floatingGeometry = workbenchGeometry(floatingCase);
    floatingGeometry.hostOverlayAudit = {
      complete: true,
      windows: [{ windowId: 'floating-panel' }],
    };
    expect(
      compareWorkbenchGeometry(
        floatingGeometry,
        floatingGeometry,
        floatingCase,
        { expectedHostOverlayIds: ['floating-panel'] },
      ).errors,
    ).toContain(
      'penpot: visible floating windows lack complete factual overlay receipts',
    );
  });

  it('rejects noncanonical authored instance step key order', () => {
    const geometry = workbenchGeometry();
    geometry.layout.semanticNodes[0].instancePath =
      '[{"sourceNodeId":"caller","sourcePath":"zircon_editor/assets/ui/editor/components/workbench/modules/workbench_window.zui"}]';
    expect(compareWorkbenchGeometry(geometry, workbenchGeometry()).errors.join(' ')).toContain(
      'invalid or duplicate semantic node',
    );
  });

  it('compares effective style exactly and rejects incomplete paint receipts', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    engine.layout.semanticNodes[0].styleInventory.properties.backgroundColor =
      'rgba(1,2,3,1)';
    expect(
      compareWorkbenchGeometry(penpot, engine).errors.join(' '),
    ).toContain('Effective style differs');

    engine.layout.semanticNodes[0].styleInventory.complete = false;
    expect(
      compareWorkbenchGeometry(penpot, engine).errors.join(' '),
    ).toContain('complete style inventory');

    const missingField = workbenchGeometry();
    delete (missingField.layout.semanticNodes[0].styleInventory.properties as Record<string, unknown>)[
      'opacity'
    ];
    expect(
      compareWorkbenchGeometry(penpot, missingField).errors.join(' '),
    ).toContain('complete style inventory');

    const unknownField = workbenchGeometry();
    (unknownField.layout.semanticNodes[0].styleInventory.properties as Record<string, unknown>)[
      'padding'
    ] = 4;
    expect(
      compareWorkbenchGeometry(penpot, unknownField).errors.join(' '),
    ).toContain('complete style inventory');

    const incompleteShadow = workbenchGeometry();
    incompleteShadow.layout.semanticNodes[0].styleInventory.properties.boxShadow.complete =
      false;
    expect(
      compareWorkbenchGeometry(penpot, incompleteShadow).errors.join(' '),
    ).toContain('complete style inventory');
  });

  it('uses authored sourceNodeId while retaining renderer nodeId for geometry', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    penpot.layout.semanticNodes[0].nodeId = 'penpot-generated-17';
    engine.layout.semanticNodes[0].nodeId = 'engine-generated-v2n17';
    expect(compareWorkbenchGeometry(penpot, engine).errors).toEqual([]);
    engine.layout.semanticNodes[0].sourceNodeId = 'other-authored-node';
    expect(compareWorkbenchGeometry(penpot, engine).errors.join(' ')).toContain(
      'Unmatched semantic node',
    );
  });

  it('requires the selected source control to resolve uniquely', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    engine.layout.semanticNodes[0].controlId = 'DifferentControl';
    expect(
      compareWorkbenchGeometry(penpot, engine).errors.join(' '),
    ).toContain('workbenchState selector');
  });

  it('requires an instance selector when authored controls repeat', () => {
    const reviewCase: LayoutReviewCase = {
      ...workbenchCase,
      data: {
        managedSceneFingerprint: managedSceneFingerprintFixture(),
        workbenchState: {
          sourcePath: workbenchSourcePath,
          controlId: 'WorkbenchWindowRoot',
        },
        workbenchPresentation: workbenchPresentation(),
      },
    };
    const repeated = workbenchGeometry(reviewCase);
    for (const geometry of [repeated]) {
      geometry.semanticAudit.expectedNodeCount = 2;
      geometry.layout.semanticNodes.push({
        ...geometry.layout.semanticNodes[0],
        nodeId: 'render-repeated',
        sourceNodeId: 'root',
        instancePath: JSON.stringify([
          { sourcePath: workbenchSourcePath, sourceNodeId: 'instance' },
        ]),
      });
    }
    const expectedSemanticNodes = [
      JSON.stringify([workbenchSourcePath, 'root', '[]']),
      JSON.stringify([
        workbenchSourcePath,
        'root',
        JSON.stringify([{ sourcePath: workbenchSourcePath, sourceNodeId: 'instance' }]),
      ]),
    ];
    expect(compareWorkbenchGeometry(repeated, repeated, reviewCase, { expectedSemanticNodes }).errors.join(' ')).toContain(
      'selector resolves to 2',
    );
  });

  it('uses path, authored ID, and instance path when components reuse IDs', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    for (const geometry of [penpot, engine]) {
      geometry.semanticAudit.expectedNodeCount = 5;
      geometry.sourceIdentityProvenance.sources.push([
        importedWorkbenchSourcePath,
        'e'.repeat(64),
      ]);
      const caller = (nodeId: string) => ({
        ...geometry.layout.semanticNodes[0],
        nodeId,
        sourceNodeId: nodeId,
        controlId: null,
        parentNodeId: 'render-root',
        parentSourcePath: workbenchSourcePath,
        parentSourceNodeId: 'root',
        parentInstancePath: '[]',
        bounds: { x: 0, y: 0, width: 10, height: 10 },
      });
      geometry.layout.semanticNodes.push(caller('left_instance'));
      geometry.layout.semanticNodes.push(caller('right_instance'));
      geometry.layout.semanticNodes.push({
        ...geometry.layout.semanticNodes[0],
        sourcePath: importedWorkbenchSourcePath,
        nodeId: 'left-button-render',
        sourceNodeId: 'button',
        instancePath: JSON.stringify([
          { sourcePath: workbenchSourcePath, sourceNodeId: 'left_instance' },
        ]),
        parentNodeId: 'left_instance',
        parentSourcePath: workbenchSourcePath,
        parentSourceNodeId: 'left_instance',
        parentInstancePath: '[]',
        controlId: null,
        bounds: { x: 12, y: 12, width: 96, height: 28 },
      });
      geometry.layout.semanticNodes.push({
        ...geometry.layout.semanticNodes[0],
        sourcePath: importedWorkbenchSourcePath,
        nodeId: 'right-button-render',
        sourceNodeId: 'button',
        instancePath: JSON.stringify([
          { sourcePath: workbenchSourcePath, sourceNodeId: 'right_instance' },
        ]),
        parentNodeId: 'right_instance',
        parentSourcePath: workbenchSourcePath,
        parentSourceNodeId: 'right_instance',
        parentInstancePath: '[]',
        controlId: null,
        bounds: { x: 12, y: 44, width: 96, height: 28 },
      });
    }
    const expectedSemanticNodes = penpot.layout.semanticNodes.map((node) =>
      JSON.stringify([node.sourcePath, node.sourceNodeId, node.instancePath]),
    );
    expect(
      compareWorkbenchGeometry(penpot, engine, workbenchCase, { expectedSemanticNodes }).errors,
    ).toEqual([]);
  });

  it('requires current source identity files and capture map provenance', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    engine.sourceIdentityProvenance.sourceMapFingerprint = [
      sourceMapPath,
      'f'.repeat(64),
    ];
    expect(
      compareWorkbenchGeometry(penpot, engine).errors.join(' '),
    ).toContain('map lacks current capture fingerprint');

    engine.sourceIdentityProvenance.sourceMapFingerprint = [
      sourceMapPath,
      sourceMapHash,
    ];
    engine.sourceIdentityProvenance.sources[0][1] = 'f'.repeat(64);
    expect(
      compareWorkbenchGeometry(penpot, engine).errors.join(' '),
    ).toContain('source identity is not current');
  });

  it('requires exact source-derived loaded media coverage and matching hashes', () => {
    const penpot = workbenchGeometry();
    const engine = workbenchGeometry();
    const mediaHash = 'b'.repeat(64);
    const expectedResourceUses = [
      {
        kind: 'vector' as const,
        path: iconPath,
        sha256: mediaHash,
        sourcePath: workbenchSourcePath,
        sourceNodeId: 'root',
        instancePath: '[]',
      },
    ];
    penpot.assetAudit.resources = [
      {
        kind: 'vector',
        path: iconPath,
        sha256: mediaHash,
        loaded: true,
        sourcePath: workbenchSourcePath,
        sourceNodeId: 'root',
        instancePath: '[]',
      },
    ];
    engine.assetAudit.resources = [
      {
        kind: 'vector',
        path: iconPath,
        sha256: mediaHash,
        loaded: true,
        sourcePath: workbenchSourcePath,
        sourceNodeId: 'root',
        instancePath: '[]',
      },
    ];
    const runtimeAssets = {
      penpot: [[iconPath, mediaHash]] as Array<
        [string, string]
      >,
      engine: [[iconPath, mediaHash]] as Array<
        [string, string]
      >,
      expectedResourceUses,
    };
    expect(
      compareWorkbenchGeometry(penpot, engine, workbenchCase, runtimeAssets)
        .errors,
    ).toEqual([]);

    runtimeAssets.engine = [];
    expect(
      compareWorkbenchGeometry(penpot, engine, workbenchCase, runtimeAssets)
        .errors.join(' '),
    ).toContain('lacks file fingerprint');

    const missing = workbenchGeometry();
    expect(
      compareWorkbenchGeometry(penpot, missing, workbenchCase, {
        penpot: [[iconPath, mediaHash]],
        engine: [[iconPath, mediaHash]],
        expectedResourceUses,
      }).errors.join(' '),
    ).toContain('missing source-derived media use');
  });

  it('derives authored icon media from case-projected source owners', () => {
    const document = {
      asset: { kind: 'view' as const, id: 'workbench', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'WorkbenchWindow',
          penpot_review_source_path: workbenchSourcePath,
          penpot_review_source_node_id: 'root',
          penpot_review_instance_path: '[]',
          children: [{ node: 'grid-icon' }],
        },
        'grid-icon': {
          component: 'Icon',
          penpot_review_source_path: importedWorkbenchSourcePath,
          penpot_review_source_node_id: 'icon-grid',
          penpot_review_instance_path: JSON.stringify([
            { sourcePath: workbenchSourcePath, sourceNodeId: 'root' },
          ]),
          props: { icon: 'grid' },
        },
      },
    };
    const inventory = deriveSourceRenderInventory(document, workbenchCase, [
      { sourcePath: workbenchSourcePath, sha256: 'c'.repeat(64) },
      { sourcePath: importedWorkbenchSourcePath, sha256: 'e'.repeat(64) },
      { sourcePath: iconPath, sha256: 'b'.repeat(64) },
    ]);
    expect(inventory.errors).toEqual([]);
    expect(inventory.resourceUses).toEqual([
      {
        kind: 'vector',
        path: iconPath,
        sha256: 'b'.repeat(64),
        sourcePath: importedWorkbenchSourcePath,
        sourceNodeId: 'icon-grid',
        instancePath: JSON.stringify([
          { sourcePath: workbenchSourcePath, sourceNodeId: 'root' },
        ]),
      },
    ]);
  });

  it('derives popup menu media only from the case-visible authored menu', () => {
    const menuPath = 'zircon_editor/assets/icons';
    const iconNames = ['plus', 'folder', 'save', 'trash', 'more'];
    const menuDocument = {
      asset: { kind: 'view' as const, id: 'workbench', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'WorkbenchWindow',
          penpot_review_source_path: workbenchSourcePath,
          penpot_review_source_node_id: 'root',
          penpot_review_instance_path: '[]',
          children: [{ node: 'menu' }],
        },
        menu: {
          component: 'ContextActionMenu',
          penpot_review_source_path: importedWorkbenchSourcePath,
          penpot_review_source_node_id: 'menu',
          penpot_review_instance_path: JSON.stringify([
            { sourcePath: workbenchSourcePath, sourceNodeId: 'root' },
          ]),
          props: {
            popup_open: false,
            menu_items: iconNames.map(
              (icon) => `Label|action=menu.item.test,icon=${icon}`,
            ),
          },
          state: { popup_open: false },
        },
      },
    };
    const fingerprints = [
      { sourcePath: workbenchSourcePath, sha256: 'c'.repeat(64) },
      { sourcePath: importedWorkbenchSourcePath, sha256: 'e'.repeat(64) },
      ...iconNames.map((icon) => ({
        sourcePath: `${menuPath}/${icon}.svg`,
        sha256: 'b'.repeat(64),
      })),
    ];
    expect(
      deriveSourceRenderInventory(menuDocument, workbenchCase, fingerprints)
        .resourceUses,
    ).toEqual([]);

    menuDocument.nodes.menu.state.popup_open = true;
    const openInventory = deriveSourceRenderInventory(
      menuDocument,
      workbenchCase,
      fingerprints,
    );
    expect(openInventory.errors).toEqual([]);
    expect(openInventory.resourceUses.map((use) => use.path)).toEqual(
      iconNames.map((icon) => `${menuPath}/${icon}.svg`).sort(),
    );
    expect(
      openInventory.resourceUses.every(
        (use) =>
          use.sourcePath === importedWorkbenchSourcePath &&
          use.sourceNodeId === 'menu',
      ),
    ).toBe(true);
  });

  it('checks exact line text, text style, and file-linked actual font faces', () => {
    const evidence = workbenchTextEvidence();
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ),
    ).toEqual([]);

    evidence.native.nodes[0].nodeId = 'different-engine-render-node';
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ),
    ).toEqual([]);
    evidence.native.nodes[0].sourceNodeId = 'different-authored-node';
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ).join(' '),
    ).toContain('Unmatched');

    evidence.native.nodes[0].sourceNodeId = 'root';

    evidence.native.nodes[0].layout.lines[0].text = 'Other';
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ).join(' '),
    ).toContain('line breaks');

    evidence.native.nodes[0].layout.lines[0].text = 'Hello';
    evidence.native.nodes[0].layout.font_weight = 500;
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ).join(' '),
    ).toContain('text style differs');
  });

  it('does not require measured text or font receipts outside the paint viewport', () => {
    const evidence = workbenchTextEvidence();
    evidence.geometry.layout.semanticNodes[0].bounds.x = 1281;
    evidence.penpot.texts = [];
    evidence.native.nodes = [];

    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
      ),
    ).toEqual([]);
  });

  it('checks text receipts in each renderer when a one-pixel geometry delta crosses the viewport edge', () => {
    const evidence = workbenchTextEvidence();
    evidence.geometry.layout.semanticNodes[0].bounds.x = 1280;
    evidence.penpot.texts = [];
    evidence.native.nodes[0].text = 'Different';
    const nativeGeometry = workbenchGeometry();
    nativeGeometry.layout.semanticNodes[0].bounds.x = 1279;

    const errors = compareTextEvidence(
      evidence.penpot,
      evidence.native,
      evidence.geometry,
      workbenchCase,
      evidence.runtimeAssets,
      nativeGeometry,
    ).join(' ');
    expect(errors).toContain('Text paint visibility differs');
    expect(errors).toContain('Rendered text differs');

    evidence.native.nodes = [];
    expect(
      compareTextEvidence(
        evidence.penpot,
        evidence.native,
        evidence.geometry,
        workbenchCase,
        evidence.runtimeAssets,
        nativeGeometry,
      ).join(' '),
    ).toContain('Unmatched or fragmented measured text');
  });
});
