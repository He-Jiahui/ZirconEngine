import { ZuiDocumentError, type ZuiNode } from './zui-document';

export interface PropertyAxisValue {
  axis: string;
  value: string;
  key: string;
  start: number;
  end: number;
}

/** Preserve source spans so editing one axis never rewrites other groups or units. */
export function parsePropertyAxes(value: string): PropertyAxisValue[] {
  const result: PropertyAxisValue[] = [];
  let axis: string | undefined;
  let tokens: RegExpMatchArray[] = [];
  const push = () => {
    if (axis && tokens.length)
      result.push({
        axis,
        value: tokens.map((token) => token[0]).join(' '),
        key: `axis-${result.length}`,
        start: tokens[0].index!,
        end: tokens.at(-1)!.index! + tokens.at(-1)![0].length,
      });
    tokens = [];
  };
  for (const token of value.matchAll(/\S+/g)) {
    if (/^[XYZW]$/.test(token[0])) {
      push();
      axis = token[0];
    } else if (axis) tokens.push(token);
  }
  push();
  return result;
}

export function propertyRowValues(node: ZuiNode) {
  const props = { ...node.props, ...node.state };
  const labelProperty = (['text', 'label'] as const).find(
    (key) => typeof props[key] === 'string' && String(props[key]).trim(),
  );
  const valueProperty = (['value_text', 'value'] as const).find(
    (key) => typeof props[key] === 'string',
  );
  if (
    !valueProperty &&
    ['value_text', 'value'].some((key) => props[key] !== undefined)
  )
    throw new ZuiDocumentError(
      `Property row ${node.control_id ?? node.component} requires a mapped scalar display value.`,
    );
  const rawValue = valueProperty ? String(props[valueProperty]) : '';
  return {
    label: {
      property: labelProperty ?? null,
      characters: labelProperty ? String(props[labelProperty]).trim() : '',
    },
    value: {
      property: valueProperty ?? null,
      characters: rawValue.trim(),
      raw: rawValue,
    },
    axes: parsePropertyAxes(rawValue),
  };
}
