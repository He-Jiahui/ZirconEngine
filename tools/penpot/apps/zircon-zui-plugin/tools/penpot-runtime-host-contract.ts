import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { Board, Context, Shape, Text, Theme } from '@penpot/plugin-types';
import { build, stop } from 'esbuild';
import { Window } from 'happy-dom';
import 'ses';

import { createSandbox } from '../../../../../third_party/penpot/plugins/libs/plugins-runtime/src/lib/create-sandbox.js';
import type { createPluginManager } from '../../../../../third_party/penpot/plugins/libs/plugins-runtime/src/lib/plugin-manager.js';
import { parseZuiDocument, zuiNodes } from '../src/bridge/zui-document.js';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_NODE,
  ZUI_ROLE_TEXT,
} from '../src/metadata.js';
import type { PluginHostMessage, PluginUiMessage } from '../src/model.js';

type PluginManager = Awaited<ReturnType<typeof createPluginManager>>;
type HostShapeType = 'board' | 'text';
type PropertyBag = Record<PropertyKey, unknown>;

interface ShapeState {
  readonly type: HostShapeType;
  proxy: Shape;
  readonly values: PropertyBag;
  readonly metadata: Map<string, string>;
  readonly children: ShapeState[];
  readonly layoutChild: PropertyBag;
  parent: ShapeState | null;
  flex: PropertyBag | null;
  grid: PropertyBag | null;
  removed: boolean;
}

interface HostSummary {
  contract: 'penpot-plugins-runtime-ses';
  semanticBoards: number;
  totalShapes: number;
  exportedFile: string;
  projectedEdits: number;
  preservedRuntimeFields: true;
}

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const fixturePath = resolve(appRoot, 'src', 'bridge', 'roundtrip-fixture.zui');

class PenpotMemoryHost {
  readonly shapes: ShapeState[] = [];
  readonly messages: PluginHostMessage[] = [];
  readonly openUiCalls: Array<{
    name: string;
    url: string;
    options: unknown;
  }> = [];

  selection: Shape[] = [];
  zoomCount = 0;
  undoBeginCount = 0;
  undoFinishCount = 0;

  readonly #messageCallbacks: Array<(message: unknown) => void> = [];
  readonly #listeners = new Map<symbol, (...params: unknown[]) => void>();
  readonly #modal: ReturnType<Window['document']['createElement']>;
  readonly #flags = mutableProxy({ naturalChildOrdering: false });
  readonly #viewport = mutableProxy(
    { center: { x: 640, y: 480 } },
    {
      zoomIntoView: () => {
        this.zoomCount += 1;
      },
    },
  );
  readonly #history = mutableProxy(
    {},
    {
      undoBlockBegin: () => {
        this.undoBeginCount += 1;
        return Symbol(`zui-import-${this.undoBeginCount}`);
      },
      undoBlockFinish: () => {
        this.undoFinishCount += 1;
      },
    },
  );

  constructor(document: Window['document']) {
    this.#modal = document.createElement('div');
    this.#modal.addEventListener('message', (event) => {
      this.messages.push(
        (event as unknown as { detail: PluginHostMessage }).detail,
      );
    });
  }

  context(): Context {
    const context = {
      version: 'host-contract',
      root: null,
      currentFile: null,
      currentPage: null,
      viewport: this.#viewport,
      flags: this.#flags,
      history: this.#history,
      library: null,
      fonts: {
        all: ['Fira Sans', 'Fira Mono'].map((fontFamily) => ({
          fontFamily,
          variants: [400, 600].map((weight) => ({
            fontWeight: String(weight),
            fontStyle: 'normal',
            fontVariantId: weight === 400 ? 'regular' : String(weight),
          })),
          applyToText: (
            text: Text,
            variant: { fontWeight: string; fontVariantId: string },
          ) => {
            text.fontFamily = fontFamily;
            text.fontId = `host-contract-${fontFamily}`;
            text.fontWeight = variant.fontWeight;
            text.fontVariantId = variant.fontVariantId;
          },
        })),
      },
      currentUser: null,
      activeUsers: [],
      theme: 'light' as Theme,
      localStorage: null,
      addListener: (
        _type: string,
        callback: (...params: unknown[]) => void,
      ) => {
        const id = Symbol('penpot-listener');
        this.#listeners.set(id, callback);
        return id;
      },
      removeListener: (id: symbol) => {
        this.#listeners.delete(id);
      },
      createBoard: () => this.createShape('board') as Board,
      createText: (characters: string) =>
        this.createShape('text', characters) as Text,
    } as unknown as Context;
    Object.defineProperty(context, 'selection', {
      enumerable: true,
      get: () => this.selection,
      set: (value: Shape[]) => {
        this.selection = [...value];
      },
    });
    return context;
  }

  manager(code: string): PluginManager {
    return {
      code,
      context: this.context(),
      manifest: {
        pluginId: 'dev.zircon.zui.host-contract',
        name: 'Zircon ZUI host contract',
        host: 'https://plugins.invalid/',
        code: 'assets/plugin.js',
        version: 2,
        permissions: ['content:read', 'content:write', 'allow:downloads'],
      },
      timeouts: new Set<ReturnType<typeof setTimeout>>(),
      intervals: new Set<ReturnType<typeof setInterval>>(),
      openModal: (name: string, url: string, options?: unknown) => {
        this.openUiCalls.push({ name, url, options });
      },
      resizeModal: () => undefined,
      getModal: () => this.#modal,
      close: () => undefined,
      registerMessageCallback: (callback: (message: unknown) => void) => {
        this.#messageCallbacks.push(callback);
      },
      sendMessage: (message: unknown) => {
        this.sendUiMessage(message as PluginUiMessage);
      },
      registerListener: (
        _type: string,
        callback: (...params: unknown[]) => void,
      ) => {
        const id = Symbol('penpot-listener');
        this.#listeners.set(id, callback);
        return id;
      },
      destroyListener: (id: symbol) => {
        this.#listeners.delete(id);
      },
    } as unknown as PluginManager;
  }

  sendUiMessage(message: PluginUiMessage): void {
    for (const callback of this.#messageCallbacks) callback(message);
  }

  stateForNode(nodeId: string): ShapeState {
    const state = this.shapes.find(
      (candidate) =>
        this.metadata(candidate, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE &&
        this.metadata(candidate, ZUI_METADATA_NODE_ID) === nodeId,
    );
    assert.ok(state, `missing semantic board for ${nodeId}`);
    return state;
  }

  textForNode(nodeId: string): ShapeState {
    const board = this.stateForNode(nodeId);
    const text = board.children.find(
      (candidate) =>
        this.metadata(candidate, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT,
    );
    assert.ok(text, `missing editable text for ${nodeId}`);
    return text;
  }

  metadata(state: ShapeState, key: string): string {
    return state.metadata.get(metadataKey(ZUI_METADATA_NAMESPACE, key)) ?? '';
  }

  private createShape(type: HostShapeType, characters = ''): Shape {
    const values: PropertyBag = {
      id: `shape-${this.shapes.length + 1}`,
      name: '',
      x: 0,
      y: 0,
      width: 100,
      height: 100,
      opacity: 1,
      borderRadius: 0,
      clipContent: false,
      fills: [],
      strokes: [],
    };
    if (type === 'text') {
      Object.assign(values, {
        characters,
        growType: 'fixed',
        fontId: '',
        fontFamily: '',
        fontVariantId: '',
        fontWeight: '400',
        fontSize: '12',
        fontStyle: 'normal',
        lineHeight: 'normal',
        letterSpacing: '0',
        textTransform: 'none',
        textDecoration: 'none',
        direction: 'ltr',
        verticalAlign: 'top',
        align: 'left',
      });
    }

    const state: ShapeState = {
      type,
      proxy: null as unknown as Shape,
      values,
      metadata: new Map(),
      children: [],
      layoutChild: mutableProxy({
        absolute: false,
        horizontalSizing: 'fix',
        verticalSizing: 'fix',
        minWidth: null,
        maxWidth: null,
        minHeight: null,
        maxHeight: null,
      }),
      parent: null,
      flex: null,
      grid: null,
      removed: false,
    };
    const target = Object.create(null) as object;
    const proxy = new Proxy(target, {
      get: (_target, property) => this.shapeProperty(state, property),
      set: (_target, property, value) => {
        state.values[property] = value;
        return true;
      },
      has: (_target, property) =>
        (property === 'children' && state.type === 'board') ||
        Reflect.has(state.values, property),
    }) as Shape;
    state.proxy = proxy;
    this.shapes.push(state);
    return proxy;
  }

  private shapeProperty(state: ShapeState, property: PropertyKey): unknown {
    if (property === 'type') return state.type;
    if (property === 'parent') return state.parent?.proxy ?? null;
    if (property === 'parentX') {
      return (
        numberValue(state.values['x']) - numberValue(state.parent?.values['x'])
      );
    }
    if (property === 'parentY') {
      return (
        numberValue(state.values['y']) - numberValue(state.parent?.values['y'])
      );
    }
    if (property === 'children') {
      return state.children.map(({ proxy }) => proxy);
    }
    if (property === 'layoutChild') {
      return state.parent?.flex || state.parent?.grid
        ? state.layoutChild
        : null;
    }
    if (property === 'flex') return state.flex;
    if (property === 'grid') return state.grid;
    if (property === 'resize') {
      return (width: number, height: number) => {
        state.values['width'] = width;
        state.values['height'] = height;
      };
    }
    if (property === 'appendChild') {
      return (child: Shape) => this.appendChild(state, child);
    }
    if (property === 'addFlexLayout') {
      return () => this.addFlexLayout(state);
    }
    if (property === 'addGridLayout') {
      return () => this.addGridLayout(state);
    }
    if (property === 'remove') {
      return () => this.removeShape(state);
    }
    if (property === 'setSharedPluginData') {
      return (namespace: string, key: string, value: string) => {
        state.metadata.set(metadataKey(namespace, key), value);
      };
    }
    if (property === 'getSharedPluginData') {
      return (namespace: string, key: string) =>
        state.metadata.get(metadataKey(namespace, key)) ?? '';
    }
    return state.values[property];
  }

  private appendChild(parent: ShapeState, child: Shape): void {
    const childState = this.shapes.find(({ proxy }) => proxy === child);
    assert.ok(childState, 'cannot append a shape from another host');
    if (childState.parent) {
      childState.parent.children.splice(
        childState.parent.children.indexOf(childState),
        1,
      );
    }
    childState.parent = parent;
    if (!parent.children.includes(childState)) parent.children.push(childState);
  }

  private addFlexLayout(state: ShapeState): PropertyBag {
    state.grid = null;
    state.flex = mutableProxy({
      dir: 'row',
      wrap: 'nowrap',
      rowGap: 0,
      columnGap: 0,
      paddingType: 'simple',
      topPadding: 0,
      rightPadding: 0,
      bottomPadding: 0,
      leftPadding: 0,
      alignItems: 'start',
      justifyContent: 'start',
    });
    return state.flex;
  }

  private addGridLayout(state: ShapeState): PropertyBag {
    state.flex = null;
    state.grid = mutableProxy(
      {
        dir: 'column',
        rowGap: 0,
        columnGap: 0,
        paddingType: 'simple',
        topPadding: 0,
        rightPadding: 0,
        bottomPadding: 0,
        leftPadding: 0,
        alignItems: 'start',
        justifyContent: 'start',
      },
      {
        addColumn: () => undefined,
        addRow: () => undefined,
        appendChild: (child: Shape) => this.appendChild(state, child),
      },
    );
    return state.grid;
  }

  private removeShape(state: ShapeState): void {
    if (state.parent) {
      const index = state.parent.children.indexOf(state);
      if (index >= 0) state.parent.children.splice(index, 1);
      state.parent = null;
    }
    state.removed = true;
  }
}

function mutableProxy(
  initial: PropertyBag,
  methods: PropertyBag = {},
): PropertyBag {
  const values = { ...initial };
  return new Proxy(Object.create(null) as object, {
    get: (_target, property) => methods[property] ?? values[property],
    set: (_target, property, value) => {
      values[property] = value;
      return true;
    },
  }) as PropertyBag;
}

function metadataKey(namespace: string, key: string): string {
  return `${namespace}\u0000${key}`;
}

function numberValue(value: unknown): number {
  return typeof value === 'number' ? value : 0;
}

async function waitFor(
  predicate: () => boolean,
  label: string,
  timeoutMs = 2_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (!predicate()) {
    if (Date.now() >= deadline)
      throw new Error(`Timed out waiting for ${label}`);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 10));
  }
}

function lastMessage<T extends PluginHostMessage['type']>(
  messages: PluginHostMessage[],
  type: T,
): Extract<PluginHostMessage, { type: T }> {
  const message = [...messages]
    .reverse()
    .find((candidate) => candidate.type === type);
  assert.ok(message, `missing ${type} host message`);
  return message as Extract<PluginHostMessage, { type: T }>;
}

async function bundlePlugin(): Promise<string> {
  try {
    const result = await build({
      entryPoints: [resolve(appRoot, 'src', 'plugin.ts')],
      bundle: true,
      write: false,
      minify: true,
      format: 'esm',
      platform: 'browser',
      tsconfig: resolve(appRoot, 'tsconfig.plugin.json'),
      logLevel: 'silent',
    });
    const output = result.outputFiles[0];
    assert.ok(output, 'esbuild did not emit the plugin bundle');
    return output.text;
  } finally {
    await stop();
  }
}

async function runHostContract(): Promise<HostSummary> {
  const [pluginCode, fixtureSource] = await Promise.all([
    bundlePlugin(),
    readFile(fixturePath, 'utf8'),
  ]);
  const browser = new Window();
  const globals = globalThis as unknown as PropertyBag;
  globals['window'] = browser;
  globals['document'] = browser.document;
  globals['CustomEvent'] = browser.CustomEvent;

  const host = new PenpotMemoryHost(browser.document);
  repairIntrinsics({
    evalTaming: 'unsafeEval',
    stackFiltering: 'verbose',
    errorTaming: 'unsafe',
    consoleTaming: 'unsafe',
    errorTrapping: 'none',
    unhandledRejectionTrapping: 'none',
  });

  // Browser timer handles are numbers. Do not expose Node Timeout objects to SES.
  const pendingTimers = new Map<number, ReturnType<typeof setTimeout>>();
  let nextTimer = 0;
  const sandbox = createSandbox(host.manager(pluginCode), {
    setTimeout: (callback: () => void, delay: number) => {
      const id = ++nextTimer;
      pendingTimers.set(
        id,
        setTimeout(() => {
          pendingTimers.delete(id);
          callback();
        }, delay),
      );
      return id;
    },
    clearTimeout: (id: number) => {
      clearTimeout(pendingTimers.get(id));
      pendingTimers.delete(id);
    },
  });
  sandbox.evaluate();

  assert.equal(host.openUiCalls.length, 1);
  assert.deepEqual(host.openUiCalls[0], {
    name: 'ZIRCON ZUI',
    url: '?theme=light',
    options: { width: 420, height: 520 },
  });
  host.sendUiMessage({ type: 'ready' });
  assert.equal(lastMessage(host.messages, 'theme').content, 'light');
  assert.equal(lastMessage(host.messages, 'selection').canExport, false);

  host.sendUiMessage({
    type: 'import-zui',
    fileName: 'penpot_roundtrip.zui',
    source: fixtureSource,
  });
  assert.equal(host.undoBeginCount, 1);
  await waitFor(
    () => host.undoFinishCount === 1,
    'settled import layout',
    60_000,
  );
  assert.equal(host.undoFinishCount, 1);
  assert.equal(host.zoomCount, 1);
  assert.equal(host.selection.length, 1);

  const selectedAsset = host.selection[0];
  assert.ok(selectedAsset, 'import did not select the asset board');
  const assetState = host.shapes.find(({ proxy }) => proxy === selectedAsset);
  assert.ok(assetState, 'selected asset is not owned by the host');
  assert.equal(host.metadata(assetState, ZUI_METADATA_ROLE), ZUI_ROLE_ASSET);

  const semanticBoards = host.shapes.filter(
    (state) => host.metadata(state, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE,
  );
  assert.equal(semanticBoards.length, 8);
  const editableTexts = host.shapes.filter(
    (state) => host.metadata(state, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT,
  );
  const detachedLanes = host.shapes.filter(
    (state) => host.metadata(state, ZUI_METADATA_ROLE) === 'detached',
  );
  assert.equal(editableTexts.length, 5);
  assert.equal(detachedLanes.length, 1);
  assert.equal(
    host.shapes.filter(({ removed }) => !removed).length,
    semanticBoards.length +
      editableTexts.length +
      detachedLanes.length +
      host.shapes.filter((state) =>
        host.metadata(state, 'linear-content-measurement'),
      ).length +
      1,
  );
  const firstAuditRequestId = 'host-audit-first';
  host.sendUiMessage({
    type: 'audit-preview-layout',
    requestId: firstAuditRequestId,
  });
  await waitFor(
    () =>
      host.messages.some(
        (message) =>
          message.type === 'preview-audit' &&
          message.requestId === firstAuditRequestId,
      ),
    'preview layout audit',
    120_000,
  );
  const previewAudit = lastMessage(host.messages, 'preview-audit');
  assert.equal(previewAudit.layoutAudit.refreshError, undefined);
  assert.equal(previewAudit.layoutAudit.totalNodes, semanticBoards.length);
  assert.equal(previewAudit.previewBoardId, selectedAsset.id);
  assert.equal(previewAudit.requestId, firstAuditRequestId);
  const auditBoardWidth = selectedAsset.width;
  (selectedAsset as Board).resize(auditBoardWidth + 11, selectedAsset.height);
  const secondAuditRequestId = 'host-audit-second';
  host.sendUiMessage({
    type: 'audit-preview-layout',
    requestId: secondAuditRequestId,
  });
  await waitFor(
    () =>
      host.messages.some(
        (message) =>
          message.type === 'preview-audit' &&
          message.requestId === secondAuditRequestId,
      ),
    'second preview layout audit',
  );
  const secondAudit = lastMessage(host.messages, 'preview-audit');
  assert.match(
    secondAudit.layoutAudit.refreshError ?? '',
    /changed geometry without a ZUI source mapping/,
  );
  assert.equal(secondAudit.requestId, secondAuditRequestId);
  assert.equal(
    secondAudit.layoutAudit.assetBounds?.width,
    auditBoardWidth + 11,
  );
  (selectedAsset as Board).resize(auditBoardWidth, selectedAsset.height);

  const root = host.stateForNode('root');
  const titleText = host.textForNode('title');
  host.sendUiMessage({ type: 'export-zui' });
  await waitFor(
    () => lastMessage(host.messages, 'status').level !== 'working',
    'settled export',
    120_000,
  );
  assert.equal(
    lastMessage(host.messages, 'export-ready').source,
    fixtureSource,
  );
  assert.match(
    lastMessage(host.messages, 'status').message,
    /0 projected edits$/,
  );
  (root.proxy as Board).borderRadius = 10;
  (titleText.proxy as Text).characters = 'Official runtime edit';
  host.selection = [titleText.proxy];

  host.sendUiMessage({ type: 'export-zui' });
  await waitFor(
    () => lastMessage(host.messages, 'status').level !== 'working',
    'edited export',
    120_000,
  );
  assert.equal(
    lastMessage(host.messages, 'status').level,
    'success',
    lastMessage(host.messages, 'status').message,
  );
  const exported = lastMessage(host.messages, 'export-ready');
  assert.equal(exported.fileName, 'penpot_roundtrip.zui');
  const sourceDocument = parseZuiDocument(fixtureSource).document;
  const exportedDocument = parseZuiDocument(exported.source).document;
  const sourceNodes = zuiNodes(sourceDocument);
  const exportedNodes = zuiNodes(exportedDocument);
  assert.equal(exportedNodes['title'].props?.['text'], 'Official runtime edit');
  assert.equal(exportedNodes['root'].props?.['corner_radius'], 10);
  assert.deepEqual(exportedNodes['root'].events, sourceNodes['root'].events);
  assert.deepEqual(
    exportedNodes['root']['zircon_extension'],
    sourceNodes['root']['zircon_extension'],
  );
  assert.deepEqual(
    exportedNodes['title'].props?.['runtime_only'],
    sourceNodes['title'].props?.['runtime_only'],
  );

  const status = lastMessage(host.messages, 'status');
  assert.equal(status.level, 'success');
  assert.match(status.message, /2 projected edits$/);
  assert.equal(
    host.messages.some(
      (message) =>
        message.type === 'status' &&
        message.level === 'error' &&
        !message.message.includes(
          'changed geometry without a ZUI source mapping',
        ),
    ),
    false,
  );

  const assertUnmappedEditRejected = async (
    description: string,
  ): Promise<void> => {
    const exportsBefore = host.messages.filter(
      (message) => message.type === 'export-ready',
    ).length;
    host.sendUiMessage({ type: 'export-zui' });
    await waitFor(
      () => lastMessage(host.messages, 'status').level !== 'working',
      'guarded export',
    );
    assert.equal(
      lastMessage(host.messages, 'status').level,
      'error',
      description,
    );
    assert.equal(
      host.messages.filter((message) => message.type === 'export-ready').length,
      exportsBefore,
      `${description} must not publish an export`,
    );
  };
  const addedText = host.context().createText('Unmapped visible text');
  assert.ok(addedText);
  (root.proxy as Board).appendChild(addedText);
  await assertUnmappedEditRejected('added untagged descendant');
  addedText.remove();
  for (const [key, value] of [
    ['x', numberValue(titleText.values['x']) + 40],
    ['width', 8],
    ['growType', 'auto-width'],
  ] as const) {
    const previous = titleText.values[key];
    titleText.values[key] = value;
    await assertUnmappedEditRejected(`unmapped semantic text ${key}`);
    titleText.values[key] = previous;
  }
  for (const [key, value] of [
    ['hidden', true],
    ['rotation', 15],
    ['flipX', true],
  ] as const) {
    const previous = root.values[key];
    root.values[key] = value;
    await assertUnmappedEditRejected(`unmapped semantic ${key}`);
    root.values[key] = previous;
  }
  host.sendUiMessage({ type: 'export-zui' });
  await waitFor(
    () => lastMessage(host.messages, 'status').level !== 'working',
    'repaired export',
  );
  assert.equal(lastMessage(host.messages, 'status').level, 'success');

  sandbox.cleanGlobalThis();
  await browser.happyDOM.abort();
  return {
    contract: 'penpot-plugins-runtime-ses',
    semanticBoards: semanticBoards.length,
    totalShapes: host.shapes.filter(({ removed }) => !removed).length,
    exportedFile: exported.fileName,
    projectedEdits: 2,
    preservedRuntimeFields: true,
  };
}

runHostContract()
  .then((summary) => {
    console.log(JSON.stringify(summary));
    process.exit(0);
  })
  .catch((error: unknown) => {
    console.error(error);
    process.exit(1);
  });
