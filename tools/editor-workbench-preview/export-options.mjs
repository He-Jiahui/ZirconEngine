const PREVIEW_SHEET_ID = "sheet";
const ID_TOKEN_SEPARATOR = /[\s,]+/u;

export function parseDesignSelection(args) {
  const ids = [];
  let captureSheet = true;
  let exportAll = false;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--all") {
      exportAll = true;
      continue;
    }
    if (arg === "--no-sheet") {
      captureSheet = false;
      continue;
    }
    if (arg === "--design" || arg === "--ids") {
      const next = args[index + 1];
      if (next && !next.startsWith("--")) {
        appendIdTokens(ids, next);
        index += 1;
      }
      continue;
    }
    if (arg.startsWith("--design=")) {
      appendIdTokens(ids, arg.slice("--design=".length));
      continue;
    }
    if (arg.startsWith("--ids=")) {
      appendIdTokens(ids, arg.slice("--ids=".length));
      continue;
    }
    if (!arg.startsWith("--")) {
      appendIdTokens(ids, arg);
    }
  }

  if (exportAll) {
    return { ids: null, captureSheet };
  }

  const normalized = ids.map((id) => id.trim()).filter(Boolean);
  return {
    ids: normalized.length ? new Set(normalized) : null,
    captureSheet,
  };
}

export function npmConfigDesignArgs(env) {
  const args = [];
  if (env.npm_config_all === "true") {
    args.push("--all");
  }
  if (env.npm_config_ids) {
    args.push(`--ids=${env.npm_config_ids}`);
  }
  if (env.npm_config_design) {
    args.push(`--design=${env.npm_config_design}`);
  }
  if (env.npm_config_sheet === "" || env.npm_config_sheet === "false" || env.npm_config_no_sheet === "true") {
    args.push("--no-sheet");
  }
  return args;
}

export function selectDesigns(designs, ids) {
  if (!ids) {
    return designs;
  }

  const selected = designs.filter((design) => ids.has(design.id) || ids.has(design.output));
  const known = new Set(designs.flatMap((design) => [design.id, design.output]));
  const unknown = [...ids].filter((id) => id !== PREVIEW_SHEET_ID && !known.has(id));
  if (unknown.length) {
    throw new Error(`unknown design id(s): ${unknown.join(", ")}`);
  }
  return selected;
}

export function shouldCapturePreviewSheet(selection) {
  return selection.captureSheet && (!selection.ids || selection.ids.has(PREVIEW_SHEET_ID));
}

function appendIdTokens(ids, value) {
  for (const token of String(value).split(ID_TOKEN_SEPARATOR)) {
    ids.push(token);
  }
}
