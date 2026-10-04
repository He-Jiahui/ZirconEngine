import { ZuiDocumentError, type ZuiNode } from './zui-document';
import { hasNativePainterSourceParts } from './zui-native-painter-projection';
import { NATIVE_PAINTER_COMPONENTS, nativePainterComponent } from './zui-native-painter-role';

// These Runtime painters own multiple visual parts. A generic owner rectangle
// cannot stand in for their content or provide editable source mappings.
const UNMAPPED_PAINTERS = NATIVE_PAINTER_COMPONENTS;

export function assertPainterMapped(
  node: ZuiNode,
  nodeId: string,
  hidden: boolean,
): void {
  const component = nativePainterComponent(node) ?? node.component;
  const variant = String(node.props?.['component_variant'] ?? '')
    .split(/\s+/)
    .find((value) =>
      ['sample-grid', 'timeline-strip', 'weight-heatmap'].includes(value),
    );
  const canvas =
    component === 'Canvas' || node.props?.['component_role'] === 'canvas';
  if (hidden || (!UNMAPPED_PAINTERS.has(component) && !(canvas && variant)))
    return;
  // These are authored runtime painters. Their owner node is the native
  // source-part mapping, provided the source carries the painter's data.
  // Keep rejecting malformed nodes so a missing mapping cannot become a
  // generic rectangle silently.
  if (hasNativePainterSourceParts(node)) return;
  const message = `Penpot visual mapping unavailable for ${component}${canvas && variant ? ` (${variant})` : ''} at nodes.${nodeId}; native content and source-part mappings are required.`;
  throw new ZuiDocumentError(message, [
    {
      severity: 'error',
      code: 'native-painter-unmapped',
      message,
      path: `nodes.${nodeId}.component`,
    },
  ]);
}
