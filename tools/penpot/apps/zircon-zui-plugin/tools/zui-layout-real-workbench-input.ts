import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { cloneZuiDocument, serializeZuiDocument, type ZuiDocument } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/zui-document';
import { LayoutDependencies } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-dependencies';
import { prepareComponentReviewHost } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-component-hosts';
import { trackedZuiPaths } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-catalog';
import { sha256 } from './zui-layout-real-session-contract.mjs';
import { validateLayoutReviewCase, type LayoutReviewCase } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-review-contract';
import { verifyWorkbenchManagedSceneFingerprint } from '../../../../../dev/penpot/plugins/apps/zircon-zui-plugin/tools/zui-layout-workbench-managed-inputs';

const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const hostSources = ['zircon_editor/assets/ui/editor/host/editor_main_frame.zui', owner,
  'zircon_editor/assets/ui/editor/unreal/editor_shell.zui',
  'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/tab.zui', theme];

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
