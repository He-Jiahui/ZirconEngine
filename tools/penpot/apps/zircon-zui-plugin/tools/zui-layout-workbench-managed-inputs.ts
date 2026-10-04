import { createHash } from 'node:crypto';
import { lstatSync, realpathSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { dirname, isAbsolute, resolve } from 'node:path';
import {
  requireApprovedPenpotArtifactDirectory,
  requireApprovedPenpotArtifactFile,
} from './zui-layout-artifact-path';
import type { LayoutReviewCase } from './zui-layout-review-contract';

export const WORKBENCH_REVIEW_PROJECT_ROOT_ENV =
  'ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT';
export const WORKBENCH_MANAGED_SCENE_SOURCES = [
  'zircon-project.toml',
  'assets/scenes/main.scene.toml',
  'assets/scenes/workbench-review-empty.scene.toml',
] as const;

export interface WorkbenchManagedSceneFingerprint {
  projectRoot: string;
  sources: Array<{ sourcePath: string; sha256: string }>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function exactFields(
  value: Record<string, unknown>,
  fields: readonly string[],
  label: string,
): void {
  if (
    Object.keys(value).length !== fields.length ||
    Object.keys(value).some((key) => !fields.includes(key))
  )
    throw new Error(`${label} has missing or unsupported fields`);
}

/** Reject old or self-selected input closures before any filesystem access. */
export function validateWorkbenchManagedSceneFingerprint(
  value: unknown,
): WorkbenchManagedSceneFingerprint {
  if (!isRecord(value))
    throw new Error('managedSceneFingerprint must be an object');
  exactFields(value, ['projectRoot', 'sources'], 'managedSceneFingerprint');
  if (
    typeof value['projectRoot'] !== 'string' ||
    !isAbsolute(value['projectRoot']) ||
    /\p{Cc}/u.test(value['projectRoot'])
  )
    throw new Error(
      'managedSceneFingerprint.projectRoot must be an absolute physical project path',
    );
  const sources = value['sources'];
  if (
    !Array.isArray(sources) ||
    sources.length !== WORKBENCH_MANAGED_SCENE_SOURCES.length
  )
    throw new Error(
      'managedSceneFingerprint must include the project manifest and both managed scene sources',
    );
  for (const [index, source] of sources.entries()) {
    if (!isRecord(source))
      throw new Error('managedSceneFingerprint source must be an object');
    exactFields(
      source,
      ['sourcePath', 'sha256'],
      'managedSceneFingerprint source',
    );
    if (source['sourcePath'] !== WORKBENCH_MANAGED_SCENE_SOURCES[index])
      throw new Error(
        'managedSceneFingerprint must use the exact ordered project-relative scene sources',
      );
    if (
      typeof source['sha256'] !== 'string' ||
      !/^[0-9a-f]{64}$/.test(source['sha256'])
    )
      throw new Error(
        'managedSceneFingerprint source sha256 must be lowercase SHA-256',
      );
  }
  return value as unknown as WorkbenchManagedSceneFingerprint;
}

function comparablePhysicalPath(value: string): string {
  const normal = value.replaceAll('/', '\\');
  const withoutDevicePrefix = normal.startsWith('\\\\?\\')
    ? normal.slice(4)
    : normal;
  return withoutDevicePrefix.replace(/[\\]+$/, '').toLowerCase();
}

/** The exporter uses canonical Windows paths, including the device prefix. */
export function assertWorkbenchManagedProjectPath(
  fingerprint: WorkbenchManagedSceneFingerprint,
  projectPath: unknown,
): void {
  if (
    typeof projectPath !== 'string' ||
    comparablePhysicalPath(projectPath) !==
      comparablePhysicalPath(fingerprint.projectRoot)
  )
    throw new Error(
      'Workbench presentation differs from its bound project root',
    );
}

function requireUnaliasedPhysicalPath(path: string): string {
  let ancestor = resolve(path);
  while (true) {
    if (lstatSync(ancestor).isSymbolicLink())
      throw new Error(
        `Managed project input path contains a reparse alias: ${path}`,
      );
    const parent = dirname(ancestor);
    if (parent === ancestor) break;
    ancestor = parent;
  }
  const actual = realpathSync.native(path);
  if (comparablePhysicalPath(actual) !== comparablePhysicalPath(path))
    throw new Error(
      `Managed project input path is not its canonical physical path: ${path}`,
    );
  return actual;
}

/** Rehash the independently bound App project, never a root selected by JSON. */
export async function verifyWorkbenchManagedSceneFingerprint(
  value: unknown,
): Promise<WorkbenchManagedSceneFingerprint> {
  const fingerprint = validateWorkbenchManagedSceneFingerprint(value);
  const configured = process.env[WORKBENCH_REVIEW_PROJECT_ROOT_ENV];
  if (!configured || !isAbsolute(configured))
    throw new Error(
      `${WORKBENCH_REVIEW_PROJECT_ROOT_ENV} must bind the actual managed review project`,
    );
  const projectRoot = requireApprovedPenpotArtifactDirectory(
    configured,
    WORKBENCH_REVIEW_PROJECT_ROOT_ENV,
  );
  if (
    comparablePhysicalPath(configured) !== comparablePhysicalPath(projectRoot)
  )
    throw new Error(
      'Managed project binding is not its canonical physical path',
    );
  const physicalRoot = requireUnaliasedPhysicalPath(projectRoot);
  if (
    comparablePhysicalPath(fingerprint.projectRoot) !==
    comparablePhysicalPath(physicalRoot)
  )
    throw new Error(
      'managedSceneFingerprint differs from the caller-bound project root',
    );
  for (const source of fingerprint.sources) {
    const sourcePath = requireApprovedPenpotArtifactFile(
      resolve(projectRoot, source.sourcePath),
      source.sourcePath,
    );
    const physicalSource = requireUnaliasedPhysicalPath(sourcePath);
    const actualHash = createHash('sha256')
      .update(await readFile(physicalSource))
      .digest('hex');
    if (actualHash !== source.sha256)
      throw new Error(
        `Managed scene input changed after product snapshot export: ${source.sourcePath}`,
      );
    if (requireUnaliasedPhysicalPath(sourcePath) !== physicalSource)
      throw new Error(
        `Managed scene physical path changed during validation: ${source.sourcePath}`,
      );
  }
  requireUnaliasedPhysicalPath(projectRoot);
  return fingerprint;
}

/** Also enforce freshness for catalogs that bypass the snapshot JSON reader. */
export async function verifyWorkbenchManagedInputCases(
  cases: readonly LayoutReviewCase[],
): Promise<void> {
  const checked = new Set<string>();
  for (const reviewCase of cases) {
    if (!Object.hasOwn(reviewCase.data, 'workbenchState')) continue;
    const fingerprint = validateWorkbenchManagedSceneFingerprint(
      reviewCase.data['managedSceneFingerprint'],
    );
    const key = JSON.stringify(fingerprint);
    if (!checked.has(key)) {
      await verifyWorkbenchManagedSceneFingerprint(fingerprint);
      checked.add(key);
    }
    const presentation = reviewCase.data['workbenchPresentation'];
    const status = isRecord(presentation) ? presentation['status'] : undefined;
    assertWorkbenchManagedProjectPath(
      fingerprint,
      isRecord(status) ? status['projectPath'] : undefined,
    );
  }
}
