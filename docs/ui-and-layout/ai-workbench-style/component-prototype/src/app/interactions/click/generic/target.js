export function genericCommandTarget(event) {
  return event.target.closest("button, .zr-menu-row");
}
