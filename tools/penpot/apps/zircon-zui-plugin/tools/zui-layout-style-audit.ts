import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { realpathSync } from 'node:fs';
import { dirname, isAbsolute, relative, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { assertLayoutCurrentRoots, createLayoutCurrentContext, layoutCurrentOutputPath, layoutCurrentSourcePath, type LayoutCurrentContext } from './zui-layout-paths';
import { assertCurrentLayoutOutputCatalog, readCurrentLayoutOutputCatalog } from './zui-layout-gallery';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import type { CatalogManifest } from './zui-layout-catalog.js';
import { canonicalSha256 } from './zui-layout-review-contract.js';
import { bytesSha256 } from './zui-layout-evidence.js';
import {
  checkZuiStyleContract,
  isStyleContractExempt,
  type StyleIssue,
} from './zui-style-contract.js';

export interface StyleAuditEntry {
  sourcePath: string;
  sourceSha256: string;
  sourceClassification: string;
  exempt: boolean;
  issueCount: number;
  issueSha256: string;
  issues: StyleIssue[];
}

export interface StyleAuditReport {
  schema: 'dev.zircon.zui.style-contract-audit';
  version: 1;
  generatedAt: string;
  inputIdentity: { repoRoot: string; catalogRoot: string; catalogSha256: string };
  sourceCount: number;
  auditSha256: string;
  summary: {
    files: number;
    exemptFiles: number;
    filesWithIssues: number;
    issues: number;
    byClassification: Record<string, { files: number; issues: number }>;
  };
  entries: StyleAuditEntry[];
}

export async function buildStyleAudit(
  manifest: CatalogManifest,
  context: LayoutCurrentContext,
  catalogSha256: string,
): Promise<StyleAuditReport> {
  assertCurrentLayoutOutputCatalog(context, manifest);
  const entries: StyleAuditEntry[] = [];
  for (const entry of manifest.entries) {
    for (const dependency of entry.dependencyFingerprints ?? []) {
      const bytes = await readFile(layoutCurrentSourcePath(context, dependency.sourcePath));
      if (bytesSha256(bytes) !== dependency.sha256)
        throw new Error(`Dependency changed after catalog generation: ${dependency.sourcePath}`);
    }
    const bytes = await readFile(layoutCurrentSourcePath(context, entry.sourcePath));
    const actual = bytesSha256(bytes);
    if (actual !== entry.sourceSha256)
      throw new Error(`Source changed after catalog generation: ${entry.sourcePath}`);
    const result = checkZuiStyleContract(bytes.toString('utf8'), entry.sourcePath);
    const issues = result.issues.map((item) => ({
      ...item,
      source: entry.sourcePath,
    }));
    entries.push({
      sourcePath: entry.sourcePath,
      sourceSha256: actual,
      sourceClassification: entry.sourceClassification,
      exempt: isStyleContractExempt(entry.sourcePath),
      issueCount: issues.length,
      issueSha256: canonicalSha256(issues),
      issues,
    });
  }
  entries.sort((a, b) => a.sourcePath.localeCompare(b.sourcePath, 'en'));
  const byClassification: Record<string, { files: number; issues: number }> = {};
  for (const entry of entries) {
    const bucket = (byClassification[entry.sourceClassification] ??= {
      files: 0,
      issues: 0,
    });
    bucket.files += 1;
    bucket.issues += entry.issueCount;
  }
  const stable = entries.map(({ sourcePath, sourceSha256, sourceClassification, exempt, issueCount, issueSha256 }) => ({
    sourcePath,
    sourceSha256,
    sourceClassification,
    exempt,
    issueCount,
    issueSha256,
  }));
  return {
    schema: 'dev.zircon.zui.style-contract-audit',
    version: 1,
    generatedAt: new Date().toISOString(),
    inputIdentity: { repoRoot: realpathSync.native(context.repoRoot), catalogRoot: realpathSync.native(context.outputRoot), catalogSha256 },
    sourceCount: manifest.sourceCount,
    auditSha256: canonicalSha256(stable),
    summary: {
      files: entries.length,
      exemptFiles: entries.filter((entry) => entry.exempt).length,
      filesWithIssues: entries.filter((entry) => entry.issueCount > 0).length,
      issues: entries.reduce((count, entry) => count + entry.issueCount, 0),
      byClassification,
    },
    entries,
  };
}

function comparable(report: StyleAuditReport): unknown {
  return {
    schema: report.schema,
    version: report.version,
    inputIdentity: report.inputIdentity,
    sourceCount: report.sourceCount,
    auditSha256: report.auditSha256,
    summary: report.summary,
    entries: report.entries,
  };
}

export interface StyleAuditContexts {
  readonly catalog: LayoutCurrentContext;
  readonly report: LayoutCurrentContext;
}

function overlap(left: string, right: string): boolean {
  const contains = (root: string, path: string): boolean => {
    const suffix = relative(realpathSync.native(root), realpathSync.native(path));
    return !isAbsolute(suffix) && suffix !== '..' && !/^\.\.[/\\]/.test(suffix);
  };
  return contains(left, right) || contains(right, left);
}

function validateContexts(contexts: StyleAuditContexts): void {
  for (const context of [contexts.catalog, contexts.report])
    assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
  assertLayoutCurrentRoots(contexts.catalog, contexts.report.repoRoot, contexts.catalog.outputRoot);
  if (overlap(contexts.catalog.outputRoot, contexts.report.outputRoot))
    throw new Error('Style report root must be disjoint from readonly catalog input');
}

/** Current catalog and checkout inputs are read-only; only the report role is writable. */
export async function runCurrentLayoutStyleAudit(
  args: readonly string[] = [],
  supplied?: StyleAuditContexts,
): Promise<StyleAuditReport> {
  let check = false;
  let reportRoot: string | undefined;
  const seen = new Set<string>();
  for (let index = 0; index < args.length; index++) {
    const flag = args[index]!;
    if (seen.has(flag)) throw new Error(`Duplicate style audit flag: ${flag}`);
    seen.add(flag);
    if (flag === '--check') check = true;
    else if (flag === '--report-root') {
      reportRoot = args[++index];
      if (!reportRoot || reportRoot.startsWith('--')) throw new Error('Missing --report-root value');
    } else throw new Error(`Unknown style audit flag: ${flag}`);
  }
  if (supplied && reportRoot !== undefined)
    throw new Error('Supplied contexts cannot be overridden by --report-root');
  if (!supplied) {
    for (const key of ['ZUI_LAYOUT_REPO_ROOT', 'ZUI_LAYOUT_OUTPUT_ROOT']) {
      const value = process.env[key];
      if (value !== undefined && (!value.trim() || !isAbsolute(value)))
        throw new Error(`Style audit requires an absolute ${key}`);
    }
    const catalog = createLayoutCurrentContext();
    reportRoot ??= process.env['ZUI_LAYOUT_STYLE_AUDIT_OUTPUT_ROOT'];
    if (!reportRoot?.trim() || !isAbsolute(reportRoot))
      throw new Error('Style audit requires an explicit absolute report root');
    supplied = { catalog, report: createLayoutCurrentContext({ repoRoot: catalog.repoRoot, outputRoot: reportRoot }) };
  }
  const contexts = supplied;
  validateContexts(contexts);
  const outputPath = (): string => layoutCurrentOutputPath(contexts.report, 'evidence/style-contract-audit.json');
  const preflight = async () => {
    validateContexts(contexts);
    outputPath();
    const { manifest, catalogSha256 } = await readCurrentLayoutOutputCatalog(contexts.catalog);
    const report = await buildStyleAudit(manifest, contexts.catalog, catalogSha256);
    return { manifest, catalogSha256, report };
  };
  const initial = await preflight();
  const verifyStable = async (): Promise<StyleAuditReport> => {
    const current = await preflight();
    if (current.catalogSha256 !== initial.catalogSha256 ||
        canonicalSha256(comparable(current.report)) !== canonicalSha256(comparable(initial.report)))
      throw new Error('Style audit catalog or sources changed during audit');
    return current.report;
  };
  if (check) {
    const existing = JSON.parse(await readFile(outputPath(), 'utf8')) as StyleAuditReport;
    const current = await verifyStable();
    if (JSON.stringify(comparable(existing)) !== JSON.stringify(comparable(current)))
      throw new Error('Style contract audit differs from current catalog or source files');
    return current;
  }
  return withLayoutCatalogLock(contexts.report.outputRoot, { command: 'zui-layout-style-audit', context: contexts.report }, async () => {
    const report = await verifyStable();
    await mkdir(dirname(outputPath()), { recursive: true });
    await verifyStable();
    await writeFile(outputPath(), `${JSON.stringify(report, null, 2)}\n`, 'utf8');
    return report;
  });
}

async function main(): Promise<void> {
  const report = await runCurrentLayoutStyleAudit(process.argv.slice(2));
  console.log(JSON.stringify({
    files: report.summary.files,
    exemptFiles: report.summary.exemptFiles,
    filesWithIssues: report.summary.filesWithIssues,
    issues: report.summary.issues,
    auditSha256: report.auditSha256,
    wrote: !process.argv.slice(2).includes('--check'),
  }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
