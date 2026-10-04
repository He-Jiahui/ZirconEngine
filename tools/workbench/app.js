const state = {
  layout: null,
  descriptors: [],
  instances: [],
  editor: null,
  openMenu: null,
  dragInstanceId: null,
  floating: [],
};

const menus = [
  ["File", ["Open Project", "Save Project", "Save Layout", "Reset Layout"]],
  ["Edit", ["Undo", "Redo"]],
  ["Selection", ["Create Cube", "Create Camera", "Create Light", "Delete"]],
  ["View", ["Hierarchy", "Inspector", "Scene", "Game", "Assets", "Prefab Editor", "Asset Browser"]],
  ["Window", ["Reset Layout"]],
  ["Help", ["Workbench Guide"]],
];

await bootstrap();
window.addEventListener("click", (event) => {
  if (!event.target.closest(".menu-bar")) {
    state.openMenu = null;
    render();
  }
});

async function bootstrap() {
  const [layout, descriptors, instances, editor] = await Promise.all([
    fetch("/fixtures/default-layout.json").then((r) => r.json()),
    fetch("/fixtures/view-descriptors.json").then((r) => r.json()),
    fetch("/fixtures/view-instances.json").then((r) => r.json()),
    fetch("/fixtures/editor-data.json").then((r) => r.json()),
  ]);
  state.layout = layout;
  state.descriptors = descriptors;
  state.instances = instances;
  state.editor = editor;
  render();
}

function render() {
  const app = document.querySelector("#app");
  app.innerHTML = "";
  const shell = el("div", "shell");
  shell.append(renderMenuBar(), renderHostStrip(), renderWorkbench(), renderStatusBar(), renderFloating());
  app.append(shell);
}

function renderMenuBar() {
  const bar = el("div", "menu-bar");
  menus.forEach(([label, items], index) => {
    const button = el("button", `menu-button ${state.openMenu === index ? "active" : ""}`);
    button.textContent = label;
    button.onclick = (event) => {
      event.stopPropagation();
      state.openMenu = state.openMenu === index ? null : index;
      render();
    };
    button.onmouseenter = () => {
      if (state.openMenu !== null && state.openMenu !== index) {
        state.openMenu = index;
        render();
      }
    };
    bar.append(button);
    if (state.openMenu === index) {
      const popup = el("div", "menu-popup");
      popup.style.left = `${10 + index * 54}px`;
      items.forEach((item) => {
        const row = el("div", "menu-item");
        row.innerHTML = `<span>${item}</span><span>${shortcutFor(item)}</span>`;
        popup.append(row);
      });
      bar.append(popup);
    }
  });
  return bar;
}

function renderHostStrip() {
  const strip = el("div", "host-strip");
  const tabs = el("div", "host-tabs");
  getPages().forEach((page) => {
    const tab = el("div", `host-tab ${page.id === state.layout.active_main_page ? "active" : ""}`);
    tab.textContent = page.title;
    tab.onclick = () => {
      state.layout.active_main_page = page.id;
      render();
    };
    tabs.append(tab);
  });
  const crumbs = el("div", "breadcrumbs");
  currentBreadcrumbs().forEach((part) => {
    const crumb = el("div", "breadcrumb");
    crumb.textContent = part;
    crumbs.append(crumb);
  });
  const floatZone = el("div", "floating-zone");
  floatZone.textContent = "Detach to Floating";
  installDropTarget(floatZone, () => {
    if (state.dragInstanceId) {
      detachToFloating(state.dragInstanceId);
      render();
    }
  });
  crumbs.append(floatZone);
  strip.append(tabs, crumbs);
  return strip;
}

function renderWorkbench() {
  const root = el("div", "workspace");
  root.append(
    renderRail("left"),
    renderDrawerColumn("left"),
    renderDocumentArea(),
    renderDrawerColumn("right"),
    renderRail("right")
  );
  return root;
}

function renderRail(side) {
  const rail = el("div", `rail ${side === "right" ? "right" : ""}`);
  const slots = side === "left" ? ["LeftTop", "LeftBottom", "BottomLeft"] : ["RightTop", "RightBottom", "BottomRight"];
  slots.forEach((slot) => {
    const drawer = state.layout.drawers[slot];
    const activeId = drawer.active_view || drawer.tab_stack.active_tab;
    const descriptor = activeId ? descriptorById(instanceById(activeId)?.descriptor_id) : null;
    const button = el("button", `rail-button ${drawer.mode !== "Collapsed" ? "active" : ""}`);
    button.title = descriptor?.default_title ?? slot;
    button.innerHTML = descriptor ? `<img src="${iconPath(descriptor.icon_key)}" alt="" />` : "•";
    button.onclick = () => {
      drawer.mode = drawer.mode === "Collapsed" ? "Pinned" : "Collapsed";
      render();
    };
    installDropTarget(button, () => {
      if (state.dragInstanceId) {
        moveToDrawer(state.dragInstanceId, slot);
        render();
      }
    });
    rail.append(button);
  });
  return rail;
}

function renderDrawerColumn(side) {
  const column = el("div", "drawer-column");
  const slots = side === "left" ? ["LeftTop", "LeftBottom"] : ["RightTop", "RightBottom"];
  slots.forEach((slot) => column.append(renderDrawer(slot)));
  return column;
}

function renderDrawer(slot) {
  const drawer = state.layout.drawers[slot];
  const panel = el("div", `drawer-panel ${drawer.mode === "Collapsed" || !isWorkbenchActive() ? "hidden" : ""}`);
  const header = el("div", "drawer-header");
  header.innerHTML = `<span>${slotLabel(slot)}</span><span>${drawer.mode}</span>`;
  panel.append(header);

  const tabs = el("div", "drawer-tabs");
  drawer.tab_stack.tabs.forEach((instanceId) => {
    const instance = instanceById(instanceId);
    const descriptor = descriptorById(instance.descriptor_id);
    const tab = el("div", `drawer-tab ${drawer.active_view === instanceId ? "active" : ""}`);
    tab.draggable = true;
    tab.innerHTML = `<img src="${iconPath(descriptor.icon_key)}" alt="" /><span>${instance.title}</span>`;
    tab.onclick = () => {
      drawer.active_view = instanceId;
      drawer.tab_stack.active_tab = instanceId;
      render();
    };
    tab.ondragstart = () => {
      state.dragInstanceId = instanceId;
    };
    tabs.append(tab);
  });
  panel.append(tabs);

  const body = el("div", "panel-body");
  installDropTarget(body, () => {
    if (state.dragInstanceId) {
      moveToDrawer(state.dragInstanceId, slot);
      render();
    }
  });
  const active = drawer.active_view ? instanceById(drawer.active_view) : null;
  body.append(active ? renderView(active) : placeholder("Drop a view here"));
  panel.append(body);
  return panel;
}

function renderDocumentArea() {
  const active = getActivePage();
  const column = el("div", "document-column");
  const container = el("div", "document-shell");
  if (active?.kind === "exclusive") {
    container.append(renderTabStack([active.view.instance_id], active.view.instance_id, active.id, [], active.view));
  } else if (active) {
    container.append(renderNode(active.document_workspace, active.id, []));
  }

  const bottom = el("div", "bottom-row");
  bottom.append(renderBottomDrawer("BottomLeft"), renderBottomDrawer("BottomRight"));

  column.append(container, bottom);
  return column;
}

function renderBottomDrawer(slot) {
  const drawer = state.layout.drawers[slot];
  const panel = el("div", "bottom-area");
  const header = el("div", "drawer-header");
  header.innerHTML = `<span>${slotLabel(slot)}</span><span>${drawer.mode}</span>`;
  const body = el("div", "panel-body");
  installDropTarget(body, () => {
    if (state.dragInstanceId) {
      moveToDrawer(state.dragInstanceId, slot);
      render();
    }
  });
  const active = drawer.active_view ? instanceById(drawer.active_view) : null;
  body.append(active ? renderView(active) : placeholder("Empty bottom panel"));
  panel.append(header, body);
  return panel;
}

function renderNode(node, pageId, path) {
  if (node.SplitNode) {
    const wrapper = el("div", `workspace-node split-${node.SplitNode.axis === "Horizontal" ? "horizontal" : "vertical"}`);
    wrapper.style.setProperty("--first-size", `${Math.round(node.SplitNode.ratio * 100)}%`);
    wrapper.append(
      renderNode(node.SplitNode.first, pageId, [...path, 0]),
      renderNode(node.SplitNode.second, pageId, [...path, 1])
    );
    return wrapper;
  }
  const tabs = node.Tabs;
  const activeId = tabs.active_tab ?? tabs.tabs[0];
  const active = instanceById(activeId);
  return renderTabStack(tabs.tabs, activeId, pageId, path, active);
}

function renderTabStack(tabIds, activeId, pageId, path, active) {
  const stack = el("div", "tab-stack");
  const strip = el("div", "tab-strip");
  tabIds.forEach((instanceId) => {
    const instance = instanceById(instanceId);
    const descriptor = descriptorById(instance.descriptor_id);
    const tab = el("div", `tab ${activeId === instanceId ? "active" : ""}`);
    tab.draggable = true;
    tab.innerHTML = `<img src="${iconPath(descriptor.icon_key)}" alt="" /><span>${instance.title}</span>`;
    tab.onclick = () => {
      focusDocumentTab(pageId, path, instanceId);
      render();
    };
    tab.ondragstart = () => {
      state.dragInstanceId = instanceId;
    };
    strip.append(tab);
  });
  installDropTarget(strip, () => {
    if (state.dragInstanceId) {
      moveToDocument(state.dragInstanceId, pageId, path);
      render();
    }
  });
  const body = el("div", "workspace-body");
  body.append(active ? renderView(active) : placeholder("Empty stack"));
  const overlay = el("div", "drop-overlay");
  ["left", "right", "top", "bottom"].forEach((edge) => {
    const zone = el("div", `drop-zone ${edge}`);
    installDropTarget(zone, () => {
      if (state.dragInstanceId) {
        splitDocument(pageId, path, edge, state.dragInstanceId);
        render();
      }
    });
    overlay.append(zone);
  });
  body.ondragenter = () => overlay.classList.add("active");
  body.ondragleave = () => overlay.classList.remove("active");
  body.append(overlay);
  stack.append(strip, body);
  return stack;
}

function renderView(instance) {
  const descriptor = descriptorById(instance.descriptor_id);
  if (!descriptor) return placeholder(instance.title);
  const card = el("div", "panel-card");
  switch (descriptor.descriptor_id) {
    case "editor.scene":
    case "editor.game":
      card.innerHTML = `<div class="viewport"></div>`;
      break;
    case "editor.hierarchy":
      state.editor.scene_entries.forEach((entry) => {
        const row = el("div", `list-row ${entry.selected ? "selected" : ""}`);
        row.style.paddingLeft = `${10 + entry.depth * 18}px`;
        row.textContent = entry.name;
        card.append(row);
      });
      break;
    case "editor.inspector":
      ["name", "parent"].forEach((field) => {
        const row = el("div", "list-row");
        row.textContent = `${field.toUpperCase()}: ${state.editor.inspector?.[field] ?? ""}`;
        card.append(row);
      });
      const transform = el("div", "list-row");
      transform.textContent = `TRANSLATION: ${(state.editor.inspector?.translation ?? []).join(", ")}`;
      card.append(transform);
      break;
    default:
      ["crate://scenes/sandbox.scene", "crate://meshes/cube.mesh", "crate://textures/checker.png"].forEach((item, index) => {
        const row = el("div", `list-row ${index === 0 ? "selected" : ""}`);
        row.textContent = item;
        card.append(row);
      });
      break;
  }
  return card;
}

function renderStatusBar() {
  const bar = el("div", "status-bar");
  bar.innerHTML = `
    <div class="status-group">
      <span>${state.editor.status_line}</span>
      <span>${state.editor.project_path}</span>
    </div>
    <div class="status-group">
      <span>${state.editor.viewport_size[0]} x ${state.editor.viewport_size[1]}</span>
      <span>WGPU preview</span>
    </div>
  `;
  return bar;
}

function renderFloating() {
  const layer = el("div", "floating-layer");
  state.floating.forEach((floating) => {
    const card = el("div", "floating-card");
    const title = el("div", "floating-title");
    title.textContent = floating.title;
    const body = el("div", "panel-body");
    body.append(renderView(instanceById(floating.instanceId)));
    card.append(title, body);
    layer.append(card);
  });
  return layer;
}

function getPages() {
  return state.layout.main_pages.map((entry) => {
    if (entry.WorkbenchPage) {
      return { kind: "workbench", ...entry.WorkbenchPage };
    }
    const page = entry.ExclusiveActivityWindowPage;
    return { kind: "exclusive", ...page, view: instanceById(page.window_instance) };
  });
}

function getActivePage() {
  return getPages().find((page) => page.id === state.layout.active_main_page) ?? getPages()[0];
}

function isWorkbenchActive() {
  return getActivePage()?.kind === "workbench";
}

function currentBreadcrumbs() {
  const active = getActivePage();
  if (!active) return [];
  if (active.kind === "exclusive") {
    return [active.title, active.view?.serializable_payload?.path ?? active.view?.title ?? "View"];
  }
  const focused = focusedView(active.document_workspace);
  return [active.title, focused?.title ?? "Scene"];
}

function focusedView(node) {
  if (node.Tabs) return instanceById(node.Tabs.active_tab ?? node.Tabs.tabs[0]);
  return focusedView(node.SplitNode.first) ?? focusedView(node.SplitNode.second);
}

function focusDocumentTab(pageId, path, instanceId) {
  const page = getPages().find((entry) => entry.id === pageId);
  const tabs = tabsNode(page.document_workspace, path);
  tabs.active_tab = instanceId;
}

function moveToDrawer(instanceId, slot) {
  detachInstance(instanceId);
  const drawer = state.layout.drawers[slot];
  if (!drawer.tab_stack.tabs.includes(instanceId)) drawer.tab_stack.tabs.push(instanceId);
  drawer.active_view = instanceId;
  drawer.tab_stack.active_tab = instanceId;
  drawer.mode = "Pinned";
}

function moveToDocument(instanceId, pageId, path) {
  detachInstance(instanceId);
  const page = getPages().find((entry) => entry.id === pageId);
  const tabs = tabsNode(page.document_workspace, path);
  if (!tabs.tabs.includes(instanceId)) tabs.tabs.push(instanceId);
  tabs.active_tab = instanceId;
}

function splitDocument(pageId, path, edge, instanceId) {
  detachInstance(instanceId);
  const page = getPages().find((entry) => entry.id === pageId);
  const node = nodeAtPath(page.document_workspace, path);
  const current = JSON.parse(JSON.stringify(node));
  const incoming = { Tabs: { tabs: [instanceId], active_tab: instanceId } };
  const axis = edge === "left" || edge === "right" ? "Horizontal" : "Vertical";
  const before = edge === "left" || edge === "top";
  replaceNode(page.document_workspace, path, {
    SplitNode: {
      axis,
      ratio: 0.5,
      first: before ? incoming : current,
      second: before ? current : incoming,
    },
  });
}

function detachToFloating(instanceId) {
  detachInstance(instanceId);
  state.floating.push({ instanceId, title: instanceById(instanceId)?.title ?? "Floating" });
}

function detachInstance(instanceId) {
  Object.values(state.layout.drawers).forEach((drawer) => {
    drawer.tab_stack.tabs = drawer.tab_stack.tabs.filter((id) => id !== instanceId);
    if (drawer.active_view === instanceId) drawer.active_view = drawer.tab_stack.tabs[0] ?? null;
    if (drawer.tab_stack.active_tab === instanceId) drawer.tab_stack.active_tab = drawer.tab_stack.tabs[0] ?? null;
  });
  getPages()
    .filter((page) => page.kind === "workbench")
    .forEach((page) => removeFromNode(page.document_workspace, instanceId));
}

function removeFromNode(node, instanceId) {
  if (node.Tabs) {
    node.Tabs.tabs = node.Tabs.tabs.filter((id) => id !== instanceId);
    if (node.Tabs.active_tab === instanceId) node.Tabs.active_tab = node.Tabs.tabs[0] ?? null;
    return;
  }
  removeFromNode(node.SplitNode.first, instanceId);
  removeFromNode(node.SplitNode.second, instanceId);
}

function tabsNode(root, path) {
  return nodeAtPath(root, path).Tabs;
}

function nodeAtPath(node, path) {
  if (!path.length) return node;
  const [head, ...tail] = path;
  return nodeAtPath(head === 0 ? node.SplitNode.first : node.SplitNode.second, tail);
}

function replaceNode(root, path, replacement) {
  if (!path.length) {
    Object.keys(root).forEach((key) => delete root[key]);
    Object.assign(root, replacement);
    return;
  }
  const parent = nodeAtPath(root, path.slice(0, -1));
  if (path[path.length - 1] === 0) parent.SplitNode.first = replacement;
  else parent.SplitNode.second = replacement;
}

function descriptorById(id) {
  return state.descriptors.find((descriptor) => descriptor.descriptor_id === id);
}

function instanceById(id) {
  return state.instances.find((instance) => instance.instance_id === id);
}

function slotLabel(slot) {
  return slot.replace(/([A-Z])/g, " $1").trim();
}

function iconPath(key) {
  return `/assets/icons/${key}.svg`;
}

function placeholder(text) {
  const node = el("div", "panel-card");
  node.textContent = text;
  return node;
}

function installDropTarget(node, onDrop) {
  node.ondragover = (event) => {
    event.preventDefault();
    node.classList.add("drag-over");
  };
  node.ondragleave = () => node.classList.remove("drag-over");
  node.ondrop = (event) => {
    event.preventDefault();
    node.classList.remove("drag-over");
    onDrop();
    state.dragInstanceId = null;
  };
}

function shortcutFor(item) {
  if (item === "Undo") return "Ctrl+Z";
  if (item === "Redo") return "Ctrl+Shift+Z";
  if (item.includes("Open")) return "Ctrl+O";
  if (item.includes("Save")) return "Ctrl+S";
  if (item === "Delete") return "Delete";
  return "";
}

function el(tag, className = "") {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}
