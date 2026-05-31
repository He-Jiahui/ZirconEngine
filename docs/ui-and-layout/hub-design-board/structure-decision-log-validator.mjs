import { existsSync, readFileSync } from "node:fs";
import {
  REFERENCE_ALIGNMENT_CHECKS,
  STRUCTURE_DECISION_LOG,
  STRUCTURE_DECISION_LOG_REQUIRED_TEXT,
} from "./board-registry.mjs";

export function validateStructureDecisionLogDocument(filePath) {
  if (!existsSync(filePath)) {
    throw new Error(`Missing structure decision log: ${filePath}`);
  }

  const content = readFileSync(filePath, "utf8");
  for (const snippet of STRUCTURE_DECISION_LOG_REQUIRED_TEXT) {
    if (!content.includes(snippet)) {
      throw new Error(`Structure decision log missing required text: ${snippet}`);
    }
  }

  const rows = parseStructureDecisionLogRows(content);
  if (rows.length !== REFERENCE_ALIGNMENT_CHECKS.length) {
    throw new Error(`Structure decision log has ${rows.length} decision rows; expected ${REFERENCE_ALIGNMENT_CHECKS.length}`);
  }

  for (let index = 0; index < REFERENCE_ALIGNMENT_CHECKS.length; index += 1) {
    const expected = REFERENCE_ALIGNMENT_CHECKS[index];
    const row = rows[index];
    if (row.check !== expected.id) {
      throw new Error(`Structure decision log row ${index + 1} check is ${row.check}; expected ${expected.id}`);
    }
    if (row.evidenceArtifact !== expected.primary_artifact) {
      throw new Error(
        `Structure decision log row ${index + 1} evidence is ${row.evidenceArtifact}; expected ${expected.primary_artifact}`,
      );
    }
    if (normalizeCell(row.targetRegion) !== expected.target_region) {
      throw new Error(
        `Structure decision log row ${index + 1} target region is ${row.targetRegion}; expected ${expected.target_region}`,
      );
    }
    if (!row.currentDecision.includes("ready for manual review") && !row.currentDecision.includes("secondary evidence")) {
      throw new Error(`Structure decision log row ${index + 1} must stay review-oriented; got ${row.currentDecision}`);
    }
    if (row.blockingCondition !== "manual-signoff-pending") {
      throw new Error(`Structure decision log row ${index + 1} blocker is ${row.blockingCondition}; expected manual-signoff-pending`);
    }
  }

  if (STRUCTURE_DECISION_LOG !== "docs/ui-and-layout/hub-design-board/STRUCTURE_DECISION_LOG.md") {
    throw new Error(`Unexpected structure decision log path: ${STRUCTURE_DECISION_LOG}`);
  }
}

function parseStructureDecisionLogRows(content) {
  const rows = [];
  for (const line of content.split(/\r?\n/)) {
    const match = line.match(/^\| `([^`]+)` \| `([^`]+)` \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$/);
    if (!match) {
      continue;
    }
    rows.push({
      check: match[1],
      evidenceArtifact: match[2],
      targetRegion: match[3].trim(),
      currentDecision: match[4].trim(),
      blockingCondition: match[5].trim(),
    });
  }
  return rows;
}

function normalizeCell(value) {
  return value.replaceAll("`", "");
}
