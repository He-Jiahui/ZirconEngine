import type { ZuiDocument, ZuiNode } from '../src/bridge/zui-document';

/** Resolve an authored callsite root after its prefab replaces the direct source key. */
export function sourceNode(
  document: ZuiDocument,
  sourcePath: string,
  sourceNodeId: string,
): ZuiNode {
  const matches = Object.entries(document.nodes ?? {}).filter(([id, node]) => {
    if (
      node['penpot_review_source_path'] === sourcePath &&
      node['penpot_review_source_node_id'] === sourceNodeId
    )
      return true;
    if (id !== sourceNodeId && !id.endsWith(`__${sourceNodeId}`)) return false;
    const path = node['penpot_review_instance_path'];
    if (typeof path !== 'string') return false;
    let steps: unknown;
    try {
      steps = JSON.parse(path);
    } catch {
      return false;
    }
    if (!Array.isArray(steps)) return false;
    const last: unknown = steps.at(-1);
    return (
      typeof last === 'object' &&
      last !== null &&
      !Array.isArray(last) &&
      (last as Record<string, unknown>)['sourcePath'] === sourcePath &&
      (last as Record<string, unknown>)['sourceNodeId'] === sourceNodeId
    );
  });
  if (matches.length !== 1)
    throw new Error(
      `Workbench projection expected one source node ${sourcePath}#${sourceNodeId}, found ${matches.length}`,
    );
  return matches[0]![1];
}

/** Prefab roots keep their caller control ID, while source identity names the prefab. */
export function sourceNodeByControl(
  document: ZuiDocument,
  sourcePath: string,
  controlId: string,
): ZuiNode {
  const matches = Object.values(document.nodes ?? {}).filter((node) => {
    if (node.control_id !== controlId) return false;
    if (node['penpot_review_source_path'] === sourcePath) return true;
    const path = node['penpot_review_instance_path'];
    if (typeof path !== 'string') return false;
    let steps: unknown;
    try {
      steps = JSON.parse(path);
    } catch {
      return false;
    }
    return (
      Array.isArray(steps) &&
      steps.some(
        (step: unknown) =>
          typeof step === 'object' &&
          step !== null &&
          !Array.isArray(step) &&
          (step as Record<string, unknown>)['sourcePath'] === sourcePath,
      )
    );
  });
  if (matches.length !== 1)
    throw new Error(
      `Workbench projection expected one source control ${sourcePath}#${controlId}, found ${matches.length}`,
    );
  return matches[0]!;
}
