import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { relative, resolve, sep } from 'node:path';
import { cloneZuiDocument, serializeZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { LayoutDependencies } from './zui-layout-dependencies';
import { prepareComponentReviewHost } from './zui-layout-component-hosts';
import { trackedZuiPaths } from './zui-layout-catalog';
import { sha256 } from './zui-layout-real-session-contract.mjs';
import { validateLayoutReviewCase, type LayoutReviewCase } from './zui-layout-review-contract';
import { verifyWorkbenchManagedSceneFingerprint } from './zui-layout-workbench-managed-inputs';
import { applyObservedWorkbenchMenuAndRunState } from './zui-layout-workbench-projection';
import {
  assertExpectedRootHash,
  assertFingerprintClosure,
  parseFingerprintList,
  REGISTERED_PRIVATE_UI_SOURCE_OWNERS,
  validateSourcePath,
  type ZuiSourceExportManifest,
  type ZuiSourceFingerprint,
} from '../src/bridge/zui-export-apply-contract';

const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const hostSources = ['zircon_editor/assets/ui/editor/host/editor_main_frame.zui', owner,
  'zircon_editor/assets/ui/editor/unreal/editor_shell.zui',
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/tab.zui', theme];

/** Match every imported owner and hash, including state-dependent dock Headers. */
export function validateRealWorkbenchExportSources(
  manifest: ZuiSourceExportManifest,
  expectedSources: readonly ZuiSourceFingerprint[],
  expectedRootSha256: string,
): number {
  return validateRealOwnerExportSources(manifest, owner, expectedSources, expectedRootSha256);
}

/** Keep the exact prepared owner and closure at the export boundary. */
export function validateRealOwnerExportSources(
  manifest: ZuiSourceExportManifest,
  expectedOwner: string,
  expectedSources: readonly ZuiSourceFingerprint[],
  expectedRootSha256: string,
): number {
  validateSourcePath(expectedOwner);
  assert.equal(manifest.rootSourcePath, expectedOwner, 'The export must retain the original root owner');
  const expected = parseFingerprintList(expectedSources, 'prepared owner sources');
  const exported = parseFingerprintList(manifest.sources, 'export owner sources');
  assertExpectedRootHash(expectedRootSha256, expected, expectedOwner);
  assertFingerprintClosure(exported, expected);
  return expected.length;
}

/** Authored source closure for a family/component; it grants no native frame authority. */
export async function prepareRealOwnerInput(
  repoRoot: string,
  sourcePath: string,
  originalSource: string,
  original: ZuiDocument,
  options: { themeSourcePath?: string; expectedSources?: readonly ZuiSourceFingerprint[] } = {},
) {
  const sourceOwner = relative(resolve(repoRoot), resolve(sourcePath)).split(sep).join('/');
  validateSourcePath(sourceOwner);
  const privateOwner = Object.hasOwn(REGISTERED_PRIVATE_UI_SOURCE_OWNERS, sourceOwner)
    ? REGISTERED_PRIVATE_UI_SOURCE_OWNERS[sourceOwner] : undefined;
  const expectedSources = options.expectedSources
    ? parseFingerprintList(options.expectedSources, 'admitted owner sources') : undefined;
  if (privateOwner) {
    assert.ok(expectedSources?.length, 'A registered private owner requires an independently pinned source closure');
    assert.equal(original.asset.kind, 'view', 'Private source kind differs from its production owner');
    assert.equal(original.asset.version, 2, 'Private source version differs from its production owner');
    assert.equal(original.asset.id, privateOwner.assetId, 'Private source asset differs from its production owner');
    assert.equal(original.root?.node, privateOwner.rootNodeId, 'Private source root differs from its production owner');
    assert.deepEqual(original.imports?.['styles'] ?? [], privateOwner.styleImports,
      'Private source style imports differ from its production owner');
    assert.deepEqual(original.imports?.['widgets'] ?? [], [], 'Private source widget imports require explicit admission');
  }
  const themeSourcePath = options.themeSourcePath ?? privateOwner?.themeSourcePath ?? theme;
  validateSourcePath(themeSourcePath);
  if (privateOwner) assert.equal(themeSourcePath, privateOwner.themeSourcePath, 'Private owner theme substitution is not permitted');
  assert.equal(sha256(await readFile(resolve(repoRoot, sourceOwner))), sha256(originalSource),
    'The source owner changed before preparation');
  if (expectedSources) assertExpectedRootHash(sha256(originalSource), expectedSources, sourceOwner);
  const dependencies = new LayoutDependencies();
  await dependencies.load(repoRoot, privateOwner
    ? expectedSources!.map(item => item.sourcePath)
    : [...new Set([...await trackedZuiPaths(), sourceOwner, themeSourcePath])]);
  const host = await prepareComponentReviewHost(repoRoot, sourceOwner, original, dependencies, themeSourcePath);
  const projection = host?.projection ?? cloneZuiDocument(original);
  if (!host) {
    assert.ok(projection.root?.node && projection.nodes?.[projection.root.node],
      'The original owner has no addressable review root');
    dependencies.embed(projection, sourceOwner, themeSourcePath);
  }
  const dependencySources = [...new Set([
    themeSourcePath,
    ...(host?.consumers.map((item) => item.sourcePath) ?? []),
    ...((projection['penpot_dependency_sources'] as string[] | undefined) ?? []),
  ])];
  const sourcePaths = [...new Set([sourceOwner,
    ...dependencySources.filter((path) => path.endsWith('.zui'))])].sort();
  for (const path of sourcePaths) validateSourcePath(path);
  const fingerprints = await Promise.all(sourcePaths.map(async (path) => {
    const loaded = dependencies.sourceFingerprint(path);
    assert.ok(loaded, `The source owner was not loaded for preparation: ${path}`);
    assert.equal(sha256(await readFile(resolve(repoRoot, path))), loaded.sha256,
      `The source changed while preparing its review closure: ${path}`);
    return loaded;
  }));
  assertExpectedRootHash(sha256(originalSource), fingerprints, sourceOwner);
  if (expectedSources) assertFingerprintClosure(fingerprints, expectedSources);
  const input = cloneZuiDocument(original);
  input['penpot_review_host'] = projection;
  input['penpot_dependency_sources'] = dependencySources;
  input['penpot_original_source'] = originalSource;
  input['penpot_original_source_path'] = sourceOwner;
  input['penpot_source_fingerprints'] = fingerprints;
  return {input, source: serializeZuiDocument(input), sourceFingerprints: fingerprints,
    reviewCase: undefined, managedPresentationPrepared: false, themeSourcePath};
}

export interface SourceOwnerGapEditTarget {
  readonly sourcePath: string;
  readonly sourceNodeId: string;
  readonly controlId: string;
  readonly instancePath: readonly [];
  readonly beforeGap: number;
  readonly afterGap: number;
}

/** Select one raw source field; this does not grant a source write or a frame claim. */
export function prepareRealOwnerGapEdit(
  sourcePath: string, original: ZuiDocument,
): SourceOwnerGapEditTarget {
  validateSourcePath(sourcePath);
  assert.equal(sourcePath, 'zircon_plugins/ai/editor/behavior_tree.zui',
    'Source gap editing is admitted only for the registered private AI owner');
  const registered = REGISTERED_PRIVATE_UI_SOURCE_OWNERS[sourcePath]!;
  assert.equal(original.asset.kind, 'view');
  assert.equal(original.asset.version, 2);
  assert.equal(original.asset.id, registered.assetId);
  assert.equal(original.root?.node, registered.rootNodeId, 'Source gap root differs from its registered owner');
  const node = original.nodes?.[registered.rootNodeId];
  assert.ok(node && typeof node.control_id === 'string' && node.control_id.length > 0,
    'Source gap root needs its original stable control ID');
  const rawContainer = node.layout?.['container'];
  assert.ok(rawContainer && typeof rawContainer === 'object' && !Array.isArray(rawContainer),
    'Source gap editing needs an explicit original container');
  const container = rawContainer as Record<string, unknown>;
  assert.ok(container['kind'] === 'HorizontalBox' || container['kind'] === 'VerticalBox',
    'Source gap editing requires one linear layout axis');
  assert.equal(node.component, container['kind'] === 'HorizontalBox' ? 'HorizontalGroup' : 'VerticalGroup');
  const gap = container['gap'];
  assert.ok(typeof gap === 'number' && Number.isFinite(gap) && gap >= 0 &&
    Number.isFinite(gap + 1) && gap + 1 > gap,
    'Source gap editing requires a finite scalar literal; tokens and bindings remain source owned');
  assert.ok(node.children && node.children.length > 1 &&
    node.children.every(child => typeof child.node === 'string' && original.nodes?.[child.node]),
    'Source gap editing needs multiple original child regions');
  return Object.freeze({ sourcePath, sourceNodeId: registered.rootNodeId, controlId: node.control_id,
    instancePath: [] as const, beforeGap: gap, afterGap: gap + 1 });
}

/** Admit the sole real Inspector delta without granting canonical source apply. */
export function validateRealOwnerGapExport(
  manifest: ZuiSourceExportManifest, expectedSources: readonly ZuiSourceFingerprint[],
  rootSha256: string, target: SourceOwnerGapEditTarget,
): number {
  assert.equal(target.sourcePath, 'zircon_plugins/ai/editor/behavior_tree.zui');
  assert.equal(target.sourceNodeId, 'behavior_tree_root');
  assert.equal(target.afterGap, target.beforeGap + 1);
  const owners = validateRealOwnerExportSources(manifest, target.sourcePath, expectedSources, rootSha256);
  assert.deepEqual(manifest.edits, [{
    exportNodeId: target.sourceNodeId, sourcePath: target.sourcePath, sourceNodeId: target.sourceNodeId,
    controlId: target.controlId, instancePath: '[]',
    fieldPath: ['layout', 'container', 'gap'], value: target.afterGap,
  }], 'The source edit export must contain exactly the admitted root scalar gap delta');
  return owners;
}

/** Use the established complete host preparation and its owning-source envelope. */
export async function prepareRealWorkbenchInput(repoRoot: string, sourcePath: string,
  originalSource: string, original: ZuiDocument, requireManagedPresentation = true,
  actualCase?: LayoutReviewCase) {
  assert.equal(resolve(repoRoot, owner), resolve(sourcePath), 'Complete workbench input must use its canonical owner');
  // The actual normal Editor observation supplies the case. A catalog default
  // cannot prove what the currently running product displays.
  const reviewCase = actualCase ? validateLayoutReviewCase(actualCase) : undefined;
  const presentation = reviewCase?.data['workbenchPresentation'] as Record<string, unknown> | undefined;
  if (requireManagedPresentation) {
    assert.ok(reviewCase && presentation, 'Provide the current actual normal Editor case');
    assert.equal(reviewCase.sourcePath, owner, 'Actual case must belong to the Unreal Level Editor');
    const evidence = reviewCase.data['normal_window_evidence'] as Record<string, unknown> | undefined;
    const publication = evidence?.['workbenchStatePublication'] as Record<string, unknown> | undefined;
    assert.equal(publication?.['authority'], 'normal-retained-workbench-projection');
    assert.equal(publication?.['observed'], true);
    assert.deepEqual(publication?.['workbenchPresentation'], presentation);
    const fingerprint = presentation['sourceFingerprint'] as Record<string, unknown>;
    assert.equal(fingerprint?.['sourcePath'], owner);
    assert.equal(fingerprint?.['sha256'], sha256(originalSource), 'Actual normal source differs from current Main');
    const managed = await verifyWorkbenchManagedSceneFingerprint(reviewCase.data['managedSceneFingerprint']);
    assert.deepEqual(publication?.['managedSceneFingerprint'], managed);
  }
  const dependencies = new LayoutDependencies();
  await dependencies.load(repoRoot, [...new Set<string>([...await trackedZuiPaths(), ...hostSources])]);
  const host = await prepareComponentReviewHost(repoRoot, owner, original, dependencies,
    reviewCase?.themeSourcePath ?? theme, presentation);
  assert.ok(host, 'Existing preparation did not produce the full authored workbench host');
  if (requireManagedPresentation || (reviewCase && Object.hasOwn(reviewCase.data, 'normal_window_evidence')))
    applyObservedWorkbenchMenuAndRunState(host.projection, reviewCase?.data['normal_window_evidence']);
  if (reviewCase) {
    const root = host.projection.nodes?.[host.projection.root!.node];
    assert.ok(root, 'Actual review host has no single addressable root');
    root.layout ??= {};
    for (const axis of ['width', 'height'] as const) {
      const value = reviewCase.viewport[axis];
      root.layout[axis] = { min: value, preferred: value, max: value, stretch: 'Fixed' };
    }
    host.projection['penpot_actual_normal_review_case'] = reviewCase;
  }
  const dependencySources = [...new Set<string>([...host.consumers.map((item) => item.sourcePath),
    ...((host.projection['penpot_dependency_sources'] as string[] | undefined) ?? [])])];
  const sourcePaths = [...new Set([hostSources[0]!, owner, ...host.consumers.map((item) => item.sourcePath),
    ...dependencySources.filter((path) => path.endsWith('.zui'))])].sort();
  const fingerprints = await Promise.all(sourcePaths.map(async (path) => ({ sourcePath: path,
    sha256: sha256(await readFile(resolve(repoRoot, path))) })));
  const input = cloneZuiDocument(original);
  input['penpot_review_host'] = host.projection;
  input['penpot_dependency_sources'] = dependencySources;
  input['penpot_original_source'] = originalSource;
  input['penpot_original_source_path'] = owner;
  input['penpot_source_fingerprints'] = fingerprints;
  return { input, source: serializeZuiDocument(input), sourceFingerprints: fingerprints, reviewCase,
    managedPresentationPrepared: Boolean(presentation) };
}
