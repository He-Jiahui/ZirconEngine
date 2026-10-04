import {
  ZuiDocumentError,
  type ZuiDocument,
  type ZuiNode,
} from './zui-document';

/** Native painters consume resolved numeric metrics and hex colors. */
export function controlProperties(
  document: ZuiDocument,
  node: ZuiNode,
  kind: string,
) {
  const resolve = (key: string): unknown => {
    let value: unknown = node.props?.[key];
    const visited = new Set<string>();
    while (typeof value === 'string' && value.startsWith('$')) {
      if (
        visited.has(value) ||
        !Object.hasOwn(document.tokens ?? {}, value.slice(1))
      )
        throw new ZuiDocumentError(`Unresolved ${kind} token ${key}: ${value}`);
      visited.add(value);
      value = document.tokens![value.slice(1)];
    }
    return value;
  };
  const number = (keys: string[], fallback: number): number => {
    for (const key of keys) {
      const value = resolve(key);
      if (value === undefined) continue;
      if (typeof value !== 'number' || !Number.isFinite(value))
        throw new ZuiDocumentError(`Unsupported ${kind} number ${key}.`);
      return value;
    }
    return fallback;
  };
  const metric = (
    keys: string[],
    fallback: number,
    positive = false,
  ): number => {
    const value = number(keys, fallback);
    if (value < 0 || (positive && value === 0))
      throw new ZuiDocumentError(`Unsupported ${kind} metric ${keys[0]}.`);
    return value;
  };
  const color = (keys: string[], fallback: string): string => {
    for (const key of keys) {
      const value = resolve(key);
      if (value === undefined) continue;
      if (
        typeof value !== 'string' ||
        !/^#[\da-f]{6}(?:[\da-f]{2})?$/i.test(value)
      )
        throw new ZuiDocumentError(
          `Unsupported ${kind} color ${key} on ${node.control_id ?? node.component}: ${JSON.stringify(value)}.`,
        );
      return value;
    }
    return fallback;
  };
  return { resolve, number, metric, color };
}
