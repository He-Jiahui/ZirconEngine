import { NAMESPACE, validateAnnotation } from '../../../zircon-zop-compiler/src/contract.mjs';

export function readAnnotation(shape) {
  const saved = shape.getSharedPluginData(NAMESPACE, 'binding');
  return saved ? JSON.parse(saved) : {};
}

export function normalizeBoard(board, contract, screen, assetPath, libraryIds) {
  const visited = new Set();
  function capture(shape, parentPath) {
    const path = `${parentPath}/${shape.name}`;
    if (visited.has(shape.id)) throw new Error(`${path}: repeated or cyclic Penpot shape`);
    visited.add(shape.id);
    const binding = readAnnotation(shape);
    validateAnnotation(binding, contract, screen, path);
    if (!['board', 'rectangle', 'text'].includes(shape.type)) throw new Error(`${path}: unsupported Penpot shape ${shape.type}`);
    const ref = shape.isComponentRoot?.() ? shape.component?.() : null;
    const componentRef = ref ? [ref.path, ref.name].filter(Boolean).join('/') : null;
    if (ref && (!componentRef.startsWith('zop/') || !libraryIds.includes(ref.libraryId))) throw new Error(`${path}: component needs a versioned zop library source`);
    const flex = shape.flex, grid = shape.grid;
    const component = binding.slot ? 'Slot' : componentRef ? componentRef.split('/')[1]
      : shape.type === 'text' ? 'Label' : flex ? (flex.dir === 'row' ? 'HorizontalGroup' : 'VerticalGroup') : 'Panel';
    const children = shape.children ?? [];
    if (shape.interactions?.length) throw new Error(`${path}: prototype interactions must use engineering command annotations`);
    if (shape.layoutChild && ['topMargin', 'rightMargin', 'bottomMargin', 'leftMargin'].some(key => shape.layoutChild[key])) throw new Error(`${path}: child margins are not supported yet`);
    if (flex && ([flex.alignContent, flex.justifyItems].some(value => value && !['start', 'stretch'].includes(value)))) throw new Error(`${path}: unsupported flex content alignment`);
    const sizing = shape.layoutChild ?? flex ?? shape;
    const dimension = (axis, size) => {
      const mode = sizing[axis === 'width' ? 'horizontalSizing' : 'verticalSizing'] ?? 'fix';
      if (!['fix', 'fill', 'auto'].includes(mode)) throw new Error(`${path}: unknown sizing ${mode}`);
      const result = { mode: { fix: 'fixed', fill: 'fill', auto: 'hug' }[mode] };
      if (mode === 'fix') result.value = size;
      for (const limit of ['min', 'max']) {
        const value = sizing[limit + (axis === 'width' ? 'Width' : 'Height')];
        if (value != null) result[limit] = value;
      }
      return result;
    };
    const layout = { kind: grid ? 'grid' : flex ? 'flex' : children.length ? 'free' : 'leaf',
      width: dimension('width', shape.width), height: dimension('height', shape.height) };
    if (flex) Object.assign(layout, { direction: flex.dir, gap: flex.dir === 'row' ? flex.columnGap : flex.rowGap,
      align: flex.alignItems ?? 'start', justify: flex.justifyContent ?? 'start', wrap: flex.wrap === 'wrap',
      padding: [flex.topPadding, flex.rightPadding, flex.bottomPadding, flex.leftPadding] });
    const fills = shape.fills;
    if (!Array.isArray(fills)) throw new Error(`${path}: mixed fills unsupported`);
    const normalizedFills = fills.map(fill => {
      if (fill.fillImage || fill.fillColorGradient) throw new Error(`${path}: images/gradients need a future library asset compiler`);
      return { color: fill.fillColor, opacity: fill.fillOpacity ?? 1 };
    });
    const strokes = (shape.strokes ?? []).map(stroke => {
      if (stroke.strokeColorGradient || stroke.strokeStyle && stroke.strokeStyle !== 'solid'
        || stroke.strokeAlignment && stroke.strokeAlignment !== 'inner') throw new Error(`${path}: only solid inner strokes are supported`);
      return { color: stroke.strokeColor, width: stroke.strokeWidth, opacity: stroke.strokeOpacity ?? 1 };
    });
    const radius = shape.borderRadius ?? 0;
    if (['borderRadiusTopLeft', 'borderRadiusTopRight', 'borderRadiusBottomLeft', 'borderRadiusBottomRight'].some(key => shape[key] != null && shape[key] !== radius)) throw new Error(`${path}: nonuniform radius unsupported`);
    const node = { id: shape.id, name: shape.name, component, layout, binding,
      geometry: { x: shape.parentX, y: shape.parentY, width: shape.width, height: shape.height },
      rotation: shape.rotation, opacity: shape.opacity, visible: shape.visible,
      radius, fills: normalizedFills, strokes, shadows: (shape.shadows ?? []).filter(shadow => !shadow.hidden),
      clip: shape.clipContent ?? false,
      tokens: Object.fromEntries(Object.entries(shape.tokens ?? {}).filter(([, name]) => typeof name === 'string' && name.length)),
      blur: shape.blur && !shape.blur.hidden ? shape.blur : null,
      absolute: shape.layoutChild?.absolute ?? false,
      children: children.map(child => capture(child, path)) };
    if (componentRef) node.component_ref = componentRef;
    if (shape.type === 'text') {
      if (shape.fontStyle && shape.fontStyle !== 'normal' || shape.fontFamily === 'mixed'
        || Number(shape.letterSpacing || '0') !== 0 || shape.textAlign && shape.textAlign !== 'left'
        || shape.textTransform && shape.textTransform !== 'none' || shape.textDecoration && shape.textDecoration !== 'none') throw new Error(`${path}: mixed/styled text unsupported`);
      const lineHeight = String(shape.lineHeight ?? '1.25');
      const ratio = lineHeight.endsWith('%') ? Number(lineHeight.slice(0, -1)) / 100 : Number(lineHeight);
      if (!(ratio > 0 && ratio <= 4)) throw new Error(`${path}: line height must be an explicit ratio or percent`);
      node.text = shape.characters;
      node.font = { family: shape.fontFamily, size: Number(shape.fontSize), weight: Number(shape.fontWeight), line_height: ratio };
    }
    return node;
  }
  return { screen, asset_path: assetPath, root: capture(board, board.name) };
}

export function captureTokens(catalog) {
  const result = {};
  for (const set of catalog.sets.filter(set => set.active)) {
    for (const token of set.tokens) {
      if (!token.name.startsWith('editor.')) continue;
      const value = token.resolvedValue;
      if (value == null) throw new Error(`token ${token.name}: unresolved value`);
      let type, converted;
      if (token.type === 'color') { type = 'color'; converted = value; }
      else if (token.type === 'fontFamilies' && Array.isArray(value) && value.length === 1) { type = 'fontFamily'; converted = value[0]; }
      else if (['dimension', 'spacing', 'sizing', 'fontSizes', 'borderRadius', 'borderWidth'].includes(token.type)) { type = 'dimension'; converted = { value, unit: 'px' }; }
      else if (['number', 'fontWeights', 'opacity'].includes(token.type)) { type = 'number'; converted = Number(value); }
      else throw new Error(`token ${token.name}: unsupported type ${token.type}`);
      result[token.name] = { $type: type, $value: converted };
    }
  }
  if (!Object.keys(result).length) throw new Error('No active editor.* foundation tokens in the local Penpot library');
  return result;
}
