import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import type { CatalogManifest } from './zui-layout-catalog.js';
import { canonicalSha256 } from './zui-layout-review-contract.js';
import { bytesSha256, containedPath } from './zui-layout-evidence.js';
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
): Promise<StyleAuditReport> {
  const entries: StyleAuditEntry[] = [];
  for (const entry of manifest.entries) {
    const bytes = await readFile(containedPath(manifest.repoRoot, entry.sourcePath));
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
    sourceCount: report.sourceCount,
    auditSha256: report.auditSha256,
    summary: report.summary,
    entries: report.entries,
  };
}

export async function writeOrCheckStyleAudit(
  manifestPath: string,
  outputPath: string,
  check: boolean,
): Promise<StyleAuditReport> {
  const manifest = JSON.parse(await readFile(resolve(manifestPath), 'utf8')) as CatalogManifest;
  const report = await buildStyleAudit(manifest);
  if (check) {
    const existing = JSON.parse(await readFile(resolve(outputPath), 'utf8')) as StyleAuditReport;
    if (JSON.stringify(comparable(existing)) !== JSON.stringify(comparable(report)))
      throw new Error('Style contract audit differs from current source files');
  } else {
    await writeFile(resolve(outputPath), `${JSON.stringify(report, null, 2)}\n`, 'utf8');
  }
  return report;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const catalogIndex = args.indexOf('--catalog');
  const outputIndex = args.indexOf('--output');
  const catalogPath = (catalogIndex >= 0 ? args[catalogIndex + 1] : undefined) ?? resolve(
    dirname(fileURLToPath(import.meta.url)),
    '../../../../../docs/_data/layout/catalog.json',
  );
  const outputPath = (outputIndex >= 0 ? args[outputIndex + 1] : undefined) ?? resolve(
    dirname(fileURLToPath(import.meta.url)),
    '../../../../../docs/_data/layout/evidence/style-contract-audit.json',
  );
  const report = await writeOrCheckStyleAudit(
    catalogPath,
    outputPath,
    args.includes('--check'),
  );
  console.log(JSON.stringify({
    files: report.summary.files,
    exemptFiles: report.summary.exemptFiles,
    filesWithIssues: report.summary.filesWithIssues,
    issues: report.summary.issues,
    auditSha256: report.auditSha256,
    wrote: !args.includes('--check'),
  }));
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
)
  void main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
