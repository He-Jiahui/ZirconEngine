const dottedActionPattern = /^[a-z0-9_]+(?:\.[a-z0-9_]+)+$/;

export function actionSegment(value, fallback = "command") {
  const segment = String(value ?? "")
    .trim()
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase()
    .replace(/['’]/g, "")
    .replace(/&/g, " and ")
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_|_$/g, "");
  return segment || fallback;
}

export function actionPath(scope, value, fallback = "command") {
  const scopePath = String(scope ?? "")
    .split(".")
    .map((part) => actionSegment(part, ""))
    .filter(Boolean)
    .join(".");
  const root = scopePath || "workbench.command";
  return `${root}.${actionSegment(value, fallback)}`;
}

export function isDottedActionId(value) {
  return dottedActionPattern.test(String(value ?? "").trim());
}

export function actionRouteKey(value) {
  const raw = String(value ?? "").trim();
  const leaf = raw.includes(".")
    ? raw.split(".").filter(Boolean).at(-1)
    : raw;
  return actionSegment(leaf).replace(/_/g, "-");
}

export function normalizeActionId(value, fallbackScope = "workbench.command") {
  const raw = String(value ?? "").trim();
  if (isDottedActionId(raw)) {
    return raw;
  }
  if (raw.includes(".")) {
    const normalizedPath = raw
      .split(".")
      .map((part) => actionSegment(part, ""))
      .filter(Boolean)
      .join(".");
    if (isDottedActionId(normalizedPath)) {
      return normalizedPath;
    }
  }
  return actionPath(fallbackScope, actionRouteKey(raw));
}
