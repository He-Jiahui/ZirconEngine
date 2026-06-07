export function drawerSurface({ tag = "aside", className, host, kind = "drawer", children }) {
  return `<${tag} class="zr-panel ${className}" data-surface="${kind}" data-panel-host="${host}">${children.join("")}</${tag}>`;
}
