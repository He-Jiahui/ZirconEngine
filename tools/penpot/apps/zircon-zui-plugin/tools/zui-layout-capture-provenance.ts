import { resolve } from 'node:path';
import { fileSha256 } from './zui-layout-evidence';
import {
  canonicalSha256,
  type LayoutRenderEvidence,
} from './zui-layout-review-contract';
import { resolvePluginBundlePath } from './zui-layout-penpot-session';

export async function captureProgramFingerprints(
  repoRoot: string,
): Promise<Array<[string, string]>> {
  const names = [
    'zui-layout-evidence.ts',
    'zui-layout-paths.ts',
    'zui-layout-visual-validation.ts',
    'zui-layout-visual-source-scope.ts',
    'zui-layout-penpot-cases.ts',
    'zui-layout-workbench-cases.ts',
    'zui-layout-workbench-managed-inputs.ts',
    'zui-layout-artifact-path.ts',
    'zui-layout-workbench-presentation.ts',
    'zui-layout-workbench-projection.ts',
    'penpot-workbench-canvas-edit.ts',
    'penpot-preview-layout-audit.ts',
    'zui-layout-download.ts',
    'zui-layout-penpot-session.ts',
    'zui-layout-screenshot.ts',
    'zui-layout-png.ts',
    'zui-layout-svg-ready.ts',
    'zui-layout-text-evidence.ts',
    'zui-layout-font-resources.ts',
    'zui-layout-penpot-receipts.ts',
    'zui-layout-source-render-inventory.ts',
    'zui-layout-semantic-parity.ts',
    'zui-layout-text-parity.ts',
    'zui-layout-browser-runtime.ts',
    'zui-layout-runtime-provenance.ts',
    'penpot-browser-navigation.ts',
    'zui-layout-capture-provenance.ts',
    'penpot-workbench-export-apply-contract.ts',
    'zui-layout-component-hosts.ts',
    'zui-layout-dependencies.ts',
    'zui-layout-review-contract.ts',
    'zui-layout-worker-pool.ts',
    'zui-layout-review-order.ts',
  ];
  const paths = [
    ...names.map(
      (name) => 'tools/penpot/apps/zircon-zui-plugin/tools/' + name,
    ),
    'tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-export-apply.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-export-apply-source.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-export-apply-sources.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-document.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/bridge/zui-review-case.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/penpot-asset-renderer.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/penpot-field-controls.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/penpot-field-controls.spec.ts',
    'tools/penpot/apps/zircon-zui-plugin/src/penpot-render-layout.ts',
  ];
  const pluginBundlePath = resolvePluginBundlePath();
  return await Promise.all(
    [
      ...paths.map((path) => [path, resolve(repoRoot, path)] as const),
      [pluginBundlePath, pluginBundlePath] as const,
    ].map(async ([path, sourcePath]) => {
      return [path, await fileSha256(sourcePath)] as [string, string];
    }),
  );
}

export function currentCaptureProgram(
  evidence: LayoutRenderEvidence[] | undefined,
  current: Array<[string, string]>,
): boolean {
  const hash = (values: Array<[string, string]>) =>
    canonicalSha256([...values].sort(([a], [b]) => a.localeCompare(b)));
  return Boolean(
    evidence?.length &&
    evidence.every(
      (item) =>
        item.captureProgramFingerprints &&
        hash(item.captureProgramFingerprints) === hash(current),
    ),
  );
}
