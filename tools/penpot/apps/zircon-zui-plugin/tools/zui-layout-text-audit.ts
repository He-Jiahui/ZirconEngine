import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderResult, type CatalogManifest } from './zui-layout-catalog';
import {
  bytesSha256,
  containedPath,
  fileSha256,
  rendererFilePath,
} from './zui-layout-evidence';
import { caseSha256 } from './zui-layout-review-contract';
import {
  captureProgramFingerprints,
  currentCaptureProgram,
} from './zui-layout-capture-provenance';
import {
  auditTextStructure,
  nativeSemanticNodes,
  nativeTextMeasurements,
  type TextStructureNode,
  type MeasuredText,
} from './zui-layout-text-structure';

const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const catalog: CatalogManifest = JSON.parse(
  await readFile(resolve(root, 'catalog.json'), 'utf8'),
);
const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(root, '../../..'));
const captureProgram = await captureProgramFingerprints(repoRoot);
const rendererHashes = new Map<string, string>();
type AuditRenderer = 'penpot' | 'engine';
type AuditCase = {
  caseId: string;
  renderer: AuditRenderer;
  rendererKind?: string;
  screenshotSha256: string;
  textSha256?: string;
  geometrySha256?: string;
  findings: ReturnType<typeof auditTextStructure>;
};
const records: Array<{
  sourcePath: string;
  report: string;
  cases: number;
  penpotCases: number;
  engineCases: number;
  findings: number;
}> = [];
let measuredCases = 0;

async function auditEvidence(
  entry: CatalogManifest['entries'][number],
  evidence: NonNullable<CatalogManifest['entries'][number]['penpotEvidence']>[number],
  renderer: AuditRenderer,
): Promise<AuditCase | undefined> {
  const reviewCase = entry.cases?.find((item) => item.id === evidence.caseId);
  if (
    !reviewCase ||
    caseSha256(reviewCase) !== evidence.caseSha256 ||
    evidence.sourceSha256 !== entry.sourceSha256 ||
    evidence.dependencySha256 !== entry.dependencySha256 ||
    (renderer === 'penpot' && !currentCaptureProgram([evidence], captureProgram)) ||
    !evidence.textPath ||
    !evidence.geometryPath ||
    !evidence.rendererPath ||
    !evidence.textSha256 ||
    !evidence.geometrySha256 ||
    !evidence.rendererSha256
  )
    return undefined;
  if (!rendererHashes.has(evidence.rendererPath))
    rendererHashes.set(
      evidence.rendererPath,
      await fileSha256(rendererFilePath(repoRoot, evidence.rendererPath)),
    );
  if (rendererHashes.get(evidence.rendererPath) !== evidence.rendererSha256)
    return undefined;
  const textBytes = await readFile(containedPath(root, evidence.textPath));
  const geometryBytes = await readFile(
    containedPath(root, evidence.geometryPath),
  );
  assert.equal(bytesSha256(textBytes), evidence.textSha256);
  assert.equal(bytesSha256(geometryBytes), evidence.geometrySha256);
  const textValue: unknown = JSON.parse(textBytes.toString());
  const geometryValue: unknown = JSON.parse(geometryBytes.toString());
  const geometry = geometryValue as {
    case?: { viewport?: { width?: number; height?: number } };
    layout?: { semanticNodes?: TextStructureNode[] };
  };
  const viewport = geometry.case?.viewport;
  if (
    !viewport ||
    !Number.isFinite(viewport.width) ||
    !Number.isFinite(viewport.height)
  )
    return undefined;
  const nodes =
    renderer === 'engine'
      ? nativeSemanticNodes(geometryValue)
      : (geometry.layout?.semanticNodes ?? []);
  const texts: MeasuredText[] =
    renderer === 'engine'
      ? nativeTextMeasurements(textValue)
      : (((textValue as { texts?: MeasuredText[] }).texts ?? []));
  const findings = auditTextStructure(nodes, texts, {
    width: viewport.width as number,
    height: viewport.height as number,
  });
  measuredCases++;
  return {
    caseId: evidence.caseId,
    renderer,
    ...(evidence.rendererKind ? { rendererKind: evidence.rendererKind } : {}),
    screenshotSha256: evidence.screenshotSha256,
    textSha256: evidence.textSha256,
    geometrySha256: evidence.geometrySha256,
    findings,
  };
}

for (const entry of catalog.entries) {
  const cases: AuditCase[] = [];
  for (const evidence of entry.penpotEvidence ?? []) {
    const audited = await auditEvidence(entry, evidence, 'penpot');
    if (audited) cases.push(audited);
  }
  for (const evidence of entry.engineEvidence ?? []) {
    const audited = await auditEvidence(entry, evidence, 'engine');
    if (audited) cases.push(audited);
  }
  if (!cases.length) continue;
  const report = {
    schema: 'dev.zircon.zui.text-structure-findings',
    version: 2,
    sourceSha256: entry.sourceSha256,
    dependencySha256: entry.dependencySha256,
    status: 'requires-visual-review',
    renderers: [...new Set(cases.map((item) => item.renderer))],
    cases,
  };
  const path = `${dirname(entry.outputPath)}/evidence/text-structure.json`;
  await writeFile(
    containedPath(root, path),
    `${JSON.stringify(report, null, 2)}\n`,
  );
  // Keep the per-entry handoff record synchronized with the measured report;
  // this does not change acceptance state and still leaves visual review
  // explicitly pending.
  await writeFile(
    containedPath(root, entry.resultPath),
    renderResult(entry),
  );
  records.push({
    sourcePath: entry.sourcePath,
    report: path,
    cases: cases.length,
    penpotCases: cases.filter((item) => item.renderer === 'penpot').length,
    engineCases: cases.filter((item) => item.renderer === 'engine').length,
    findings: cases.reduce((sum, item) => sum + item.findings.length, 0),
  });
}
await writeFile(
  resolve(root, 'evidence/text-structure-summary.json'),
  `${JSON.stringify({ schema: 'dev.zircon.zui.text-structure-summary', version: 2, generatedAt: new Date().toISOString(), measuredCases, entries: records, acceptanceStatus: 'unaccepted' }, null, 2)}\n`,
);
console.log(
  JSON.stringify({
    entries: records.length,
    measuredCases,
    penpotCases: records.reduce((sum, entry) => sum + entry.penpotCases, 0),
    engineCases: records.reduce((sum, entry) => sum + entry.engineCases, 0),
    entriesWithFindings: records.filter((entry) => entry.findings > 0).length,
  }),
);
