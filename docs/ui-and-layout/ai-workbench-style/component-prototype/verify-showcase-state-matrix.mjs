import { readFileSync } from "node:fs";

const repoRoot = new URL("../../../../", import.meta.url);
const sources = {
  root: readRepo("zircon_editor/assets/ui/editor/component_showcase.zui"),
  commandToolbar: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_command_toolbar.zui"),
  bottomLog: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_bottom_log.zui"),
  categoryNav: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_category_nav.zui"),
  statePanel: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_state_panel.zui"),
  visual: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_visual_section.zui"),
  input: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_input_section.zui"),
  selection: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui"),
  collections: readRepo("zircon_editor/assets/ui/editor/components/showcase/showcase_collections_section.zui"),
  categories: readRepo("zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs"),
  defaults: readRepo("zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs"),
  showcaseDemoState: readRepo("zircon_editor/src/ui/template_runtime/showcase_demo_state.rs"),
  showcaseBindings: readRepo("zircon_editor/src/ui/template_runtime/builtin/showcase_template_bindings.rs"),
  showcaseEventInputs: readRepo("zircon_editor/src/ui/retained_host/app/showcase_event_inputs.rs"),
  showcaseActions: readRepoMany([
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/action_buttons.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/binding_ids.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/commit_action.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/drag_actions.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/edit_action.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/showcase_actions/primary_action.rs",
  ]),
  viewProjection: readRepo("zircon_editor/src/ui/layouts/views/view_projection.rs"),
  retainedHostTest: readRepo("zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs"),
  categoryTest: readRepo("zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs"),
};

const showcaseSections = [
  { name: "visual", source: sources.visual },
  { name: "input", source: sources.input },
  { name: "selection", source: sources.selection },
  { name: "collections", source: sources.collections },
];
const showcaseAssets = [
  { name: "command_toolbar", source: sources.commandToolbar },
  { name: "bottom_log", source: sources.bottomLog },
  { name: "category_nav", source: sources.categoryNav },
  { name: "state_panel", source: sources.statePanel },
  ...showcaseSections,
];
const showcase = [sources.root, ...showcaseSections.map(({ source }) => source)].join("\n");
const demoNodes = showcaseSections
  .flatMap(({ name, source }) => readDemoNodes(source, name))
  .sort((left, right) => left.controlId.localeCompare(right.controlId));
const authoredEvents = showcaseAssets.flatMap(({ name, source }) => readShowcaseEvents(source, name));
const authoredEventById = new Map(authoredEvents.map((event) => [event.id, event]));
const rustBindings = showcaseBindingsFromRust(sources.showcaseBindings);
const rustBindingById = new Map(rustBindings.map((binding) => [binding.id, binding]));
const rustBindingActionIds = new Set(rustBindings.map(({ id }) => showcaseActionIdForBindingId(id)));
const demoControlIds = [
  ...new Set(demoNodes.map(({ controlId }) => controlId)),
].sort();
const componentIdMappings = sourceBetween(
  sources.defaults,
  "pub(super) fn component_id_for_control",
  "pub(super) fn default_state_for_control",
);
const componentMappingByControlId = componentMappingsFromRust(componentIdMappings);
const categoryMappingByControlId = categoryMappingsFromRust(
  sourceFrom(sources.categories, "fn demo_category_for_control"),
);
const missingCategoryMappings = demoControlIds.filter(
  (controlId) => !categoryMappingByControlId.has(controlId),
);
const missingComponentMappings = demoControlIds.filter(
  (controlId) => !componentMappingByControlId.has(controlId),
);
const staleCategoryMappings = [...categoryMappingByControlId.keys()]
  .filter((controlId) => !demoControlIds.includes(controlId))
  .sort();
const staleComponentMappings = [...componentMappingByControlId.keys()]
  .filter((controlId) => !demoControlIds.includes(controlId))
  .sort();
const mismatchedAuthoredComponents = demoNodes.filter(
  ({ controlId, component }) =>
    componentMappingByControlId.has(controlId) &&
    componentMappingByControlId.get(controlId) !== component,
);
const categoriesBySection = new Map([
  ["visual", new Set(["CATEGORY_VISUAL", "CATEGORY_FEEDBACK"])],
  ["input", new Set(["CATEGORY_INPUT", "CATEGORY_NUMERIC"])],
  ["selection", new Set(["CATEGORY_SELECTION", "CATEGORY_REFERENCE"])],
  ["collections", new Set(["CATEGORY_COLLECTIONS"])],
]);
const categoryConstants = categoryConstantsFromRust(sources.categories);
const categoryConstantValues = new Set(categoryConstants.map(({ value }) => value));
const navCategoryByControlId = navCategoryMappingsFromRust(sources.categories, categoryConstants);
const navCategoryControlIds = categoryNavControlIdsFromAsset(sources.categoryNav);
const navCategories = new Set(navCategoryByControlId.values());
const mismatchedSectionCategories = demoNodes.filter(({ section, controlId }) => {
  const allowedCategories = categoriesBySection.get(section);
  const mappedCategory = categoryMappingByControlId.get(controlId);
  return Boolean(mappedCategory && allowedCategories && !allowedCategories.has(mappedCategory));
});
const selectCategoryBindings = rustBindings.filter(({ action }) => actionKind(action) === "SelectCategory");
const selectCategoryBindingByControlId = new Map(
  selectCategoryBindings.map((binding) => [binding.controlId, binding]),
);
const missingNavCategories = categoryConstants
  .map(({ value }) => value)
  .filter((category) => !navCategories.has(category));
const unknownNavCategories = [...navCategories].filter((category) => !categoryConstantValues.has(category));
const missingCategoryNavNodes = [...navCategoryByControlId.keys()].filter(
  (controlId) => !navCategoryControlIds.has(controlId),
);
const unmappedCategoryNavNodes = [...navCategoryControlIds].filter(
  (controlId) => !navCategoryByControlId.has(controlId),
);
const missingSelectCategoryBindings = [...navCategoryByControlId.entries()].filter(
  ([controlId]) => !selectCategoryBindingByControlId.has(controlId),
);
const staleSelectCategoryBindings = selectCategoryBindings.filter(
  ({ controlId }) => !navCategoryByControlId.has(controlId),
);
const mismatchedSelectCategoryBindings = selectCategoryBindings.filter(({ controlId, action }) => {
  const category = navCategoryByControlId.get(controlId);
  return Boolean(category && category !== action.split(".")[1]);
});
const missingRustBindings = authoredEvents.filter(({ id }) => !rustBindingById.has(id));
const extraRustBindings = rustBindings.filter(({ id }) => !authoredEventById.has(id));
const mismatchedRustBindings = authoredEvents.filter(({ id, controlId, event }) => {
  const binding = rustBindingById.get(id);
  return Boolean(binding && (binding.controlId !== controlId || binding.event !== event));
});
const mismatchedShowcaseEventRoutes = authoredEvents.filter(({ id, route }) => {
  const binding = rustBindingById.get(id);
  return Boolean(binding && route !== showcaseRouteForAction(binding.action));
});
const nonCanonicalEventNamespaces = authoredEvents.filter(
  ({ id, route }) => !id.startsWith("UiComponentShowcase/") || !route.startsWith("component_lab.showcase."),
);
const supportedShowcaseActionKinds = showcaseActionKindsFromRuntime(sources.showcaseDemoState);
const unsupportedShowcaseBindingActions = rustBindings.filter(
  ({ action }) => !supportedShowcaseActionKinds.has(actionKind(action)),
);
const requiredTypedActionInputs = new Map([
  ["Hover", "Hover"],
  ["Press", "Press"],
  ["DropHover", "DropHover"],
  ["ActiveDragTarget", "ActiveDragTarget"],
]);
const missingTypedShowcaseActionInputs = rustBindings
  .filter(({ action }) => requiredTypedActionInputs.has(actionKind(action)))
  .filter(
    ({ id, action }) =>
      !hasShowcaseActionInputMapping(
        sources.showcaseEventInputs,
        showcaseBindingSuffix(id),
        requiredTypedActionInputs.get(actionKind(action)),
      ),
  );
const requiredCollectionValueInputs = new Set(["ArrayFieldChanged", "MapFieldChanged"]);
const missingCollectionValueActionInputs = rustBindings
  .filter(({ id }) => requiredCollectionValueInputs.has(showcaseBindingSuffix(id)))
  .filter(
    ({ id }) =>
      !hasShowcaseActionInputMapping(
        sources.showcaseEventInputs,
        showcaseBindingSuffix(id),
        "Value",
      ),
  );
const requiredDragDeltaInputs = new Map([
  ["DragDelta", "DragDelta"],
  ["LargeDragDelta", "LargeDragDelta"],
]);
const missingDragDeltaActionInputs = rustBindings
  .filter(({ action }) => requiredDragDeltaInputs.has(actionKind(action)))
  .filter(
    ({ id, action }) =>
      !hasShowcaseActionInputMapping(
        sources.showcaseEventInputs,
        showcaseBindingSuffix(id),
        requiredDragDeltaInputs.get(actionKind(action)),
      ),
  );
const requiredShowcaseActionInputChoices = new Map([
  ["ValueChanged", [variantInput("Value")]],
  ["Change", [variantInput("Toggle")]],
  ["Hover", [variantInput("Hover")]],
  ["Press", [variantInput("Press")]],
  ["DragDelta", [variantInput("DragDelta")]],
  ["LargeDragDelta", [variantInput("LargeDragDelta")]],
  ["OpenPopupAt", [variantInput("OpenPopupAt")]],
  ["SelectOption", [helperInput("select_option"), variantInput("SelectOption")]],
  ["DropReference", [variantInput("DropReference")]],
  ["DropHover", [variantInput("DropHover")]],
  ["ActiveDragTarget", [variantInput("ActiveDragTarget")]],
  ["ToggleExpanded", [variantInput("Toggle")]],
  ["AddElement", [variantInput("AddElement")]],
  ["SetElement", [variantInput("SetElement")]],
  ["RemoveElement", [variantInput("RemoveElement")]],
  ["MoveElement", [variantInput("MoveElement")]],
  ["AddMapEntry", [variantInput("AddMapEntry")]],
  ["SetMapEntry", [variantInput("SetMapEntry"), variantInput("RenameMapEntry")]],
  ["RemoveMapEntry", [variantInput("RemoveMapEntry")]],
  ["SetVisibleRange", [variantInput("SetVisibleRange")]],
  ["SetPage", [variantInput("SetPage")]],
  ["SetWorldTransform", [variantInput("SetWorldTransform")]],
  ["SetWorldSurface", [variantInput("SetWorldSurface")]],
]);
const missingRequiredShowcaseActionInputs = rustBindings
  .filter(({ action }) => requiredShowcaseActionInputChoices.has(actionKind(action)))
  .filter(
    ({ id, action }) =>
      !hasShowcaseActionInputChoice(
        sources.showcaseEventInputs,
        showcaseBindingSuffix(id),
        requiredShowcaseActionInputChoices.get(actionKind(action)),
      ),
  );
const retainedHostActionSuffixes = retainedHostActionSuffixesFromRust(sources.showcaseActions);
const missingRetainedHostActionBindings = retainedHostActionSuffixes.filter(
  (suffix) => !rustBindingById.has(`UiComponentShowcase/${suffix}`),
);
const staleShowcaseActionInputMatchers = staleShowcaseActionInputMatchersFromRust(
  sources.showcaseEventInputs,
  rustBindings,
);
const retainedHostTestShowcaseActionIds = showcaseActionIdsFromSource(sources.retainedHostTest);
const missingRetainedHostTestActionBindings = retainedHostTestShowcaseActionIds.filter(
  (actionId) => !rustBindingActionIds.has(actionId),
);
const rustTestShowcaseBindingIds = showcaseBindingIdsFromSources([
  sources.retainedHostTest,
  sources.categoryTest,
]);
const missingRustTestShowcaseBindings = rustTestShowcaseBindingIds.filter(
  (bindingId) => !rustBindingById.has(bindingId),
);
const categoryFilterTestBindingIds = new Set(showcaseBindingIdsFromSources([sources.categoryTest]));
const categoryFilterTestDemoControlIds = new Set(demoControlIdsFromSource(sources.categoryTest));
const missingCategoryFilterDemoControlReferences = demoControlIds.filter(
  (controlId) => !categoryFilterTestDemoControlIds.has(controlId),
);
const expectedCategoryFilterGroupByCategoryConstant = new Map([
  ["CATEGORY_VISUAL", "SHOWCASE_VISUAL_DEMO_CONTROL_IDS"],
  ["CATEGORY_FEEDBACK", "SHOWCASE_FEEDBACK_DEMO_CONTROL_IDS"],
  ["CATEGORY_INPUT", "SHOWCASE_INPUT_DEMO_CONTROL_IDS"],
  ["CATEGORY_NUMERIC", "SHOWCASE_NUMERIC_DEMO_CONTROL_IDS"],
  ["CATEGORY_SELECTION", "SHOWCASE_SELECTION_DEMO_CONTROL_IDS"],
  ["CATEGORY_REFERENCE", "SHOWCASE_REFERENCE_DEMO_CONTROL_IDS"],
  ["CATEGORY_COLLECTIONS", "SHOWCASE_COLLECTION_DEMO_CONTROL_IDS"],
]);
const expectedCategoryByCategoryFilterGroup = new Map(
  [...expectedCategoryFilterGroupByCategoryConstant.entries()].map(([category, groupName]) => [
    groupName,
    category,
  ]),
);
const expectedCategoryFilterGroupByCategory = new Map([
  ["All", "assert_all_demo_controls_visible"],
  ["Visual", "SHOWCASE_VISUAL_DEMO_CONTROL_IDS"],
  ["Feedback", "SHOWCASE_FEEDBACK_DEMO_CONTROL_IDS"],
  ["Input", "SHOWCASE_INPUT_DEMO_CONTROL_IDS"],
  ["Numeric", "SHOWCASE_NUMERIC_DEMO_CONTROL_IDS"],
  ["Selection", "SHOWCASE_SELECTION_DEMO_CONTROL_IDS"],
  ["Reference", "SHOWCASE_REFERENCE_DEMO_CONTROL_IDS"],
  ["Collections", "SHOWCASE_COLLECTION_DEMO_CONTROL_IDS"],
]);
const categoryFilterDemoGroups = categoryFilterDemoGroupsFromRust(sources.categoryTest);
const categoryFilterAggregateDemoGroups = categoryFilterAggregateDemoGroupsFromRust(
  sources.categoryTest,
);
const categoryFilterAggregateDemoGroupSet = new Set(categoryFilterAggregateDemoGroups);
const missingCategoryFilterDemoGroups = [...expectedCategoryFilterGroupByCategoryConstant.values()]
  .filter((groupName) => !categoryFilterDemoGroups.has(groupName));
const unknownCategoryFilterDemoGroups = [...categoryFilterDemoGroups.keys()]
  .filter((groupName) => !expectedCategoryByCategoryFilterGroup.has(groupName));
const missingCategoryFilterAggregateDemoGroups = [...expectedCategoryFilterGroupByCategoryConstant.values()]
  .filter((groupName) => !categoryFilterAggregateDemoGroupSet.has(groupName));
const unknownCategoryFilterAggregateDemoGroups = categoryFilterAggregateDemoGroups
  .filter((groupName) => !expectedCategoryByCategoryFilterGroup.has(groupName));
const duplicateCategoryFilterAggregateDemoGroups = duplicateValues(categoryFilterAggregateDemoGroups);
const staleCategoryFilterDemoGroupReferences = [...categoryFilterDemoGroups.entries()]
  .flatMap(([groupName, controlIds]) =>
    controlIds
      .filter((controlId) => !demoControlIds.includes(controlId))
      .map((controlId) => ({ groupName, controlId })),
  );
const duplicateCategoryFilterDemoGroupReferences =
  duplicateCategoryFilterDemoGroupReferencesFromGroups(categoryFilterDemoGroups);
const mismatchedCategoryFilterDemoGroupCategories = [...categoryFilterDemoGroups.entries()]
  .flatMap(([groupName, controlIds]) => {
    const expectedCategory = expectedCategoryByCategoryFilterGroup.get(groupName);
    if (!expectedCategory) {
      return [];
    }
    return controlIds
      .filter((controlId) => categoryMappingByControlId.get(controlId) !== expectedCategory)
      .map((controlId) => ({
        groupName,
        controlId,
        expectedCategory,
        actualCategory: categoryMappingByControlId.get(controlId) ?? "<missing>",
      }));
  });
const missingCategoryFilterSelectBindings = selectCategoryBindings
  .map(({ id }) => id)
  .filter((bindingId) => !categoryFilterTestBindingIds.has(bindingId));
const categoryFilterBlocksByBindingId = new Map(
  selectCategoryBindings.map(({ id }) => [id, categoryFilterTestBlock(sources.categoryTest, id)]),
);
const missingCategoryFilterProjectionBlocks = selectCategoryBindings.filter(
  ({ id }) => !categoryFilterBlocksByBindingId.get(id),
);
const missingCategoryFilterSelectedAssertions = selectCategoryBindings.filter(({ id, controlId }) => {
  const block = categoryFilterBlocksByBindingId.get(id);
  return !block || !hasCategorySelectedAssertion(block, controlId, true);
});
const missingCategoryFilterDeselectedAssertions = selectCategoryBindings.flatMap(({ id, controlId }) => {
  const block = categoryFilterBlocksByBindingId.get(id);
  if (!block) {
    return [{ id, controlId, missingControlId: "*" }];
  }
  return [...navCategoryControlIds]
    .filter((otherControlId) => otherControlId !== controlId)
    .filter((otherControlId) => !hasCategorySelectedAssertion(block, otherControlId, false))
    .map((missingControlId) => ({ id, controlId, missingControlId }));
});
const categoryFilterSelectedHelperComplete = categorySelectedHelperCoversNavControls(
  sources.categoryTest,
  navCategoryControlIds,
);
const missingCategoryFilterGroupAssertions = selectCategoryBindings.filter(({ id, action }) => {
  const expectedGroup = expectedCategoryFilterGroupByCategory.get(action.split(".")[1]);
  const block = categoryFilterBlocksByBindingId.get(id);
  if (!block || !expectedGroup) {
    return true;
  }
  if (expectedGroup === "assert_all_demo_controls_visible") {
    return !/assert_all_demo_controls_visible\(&host_projection\)/.test(block);
  }
  return !block.includes(`assert_demo_controls_for_category(&host_projection, ${expectedGroup})`);
});

const checks = [
  ["root imports visual section", sources.root.includes("showcase_visual_section.zui#ShowcaseVisualSection")],
  ["root imports input section", sources.root.includes("showcase_input_section.zui#ShowcaseInputSection")],
  ["root imports selection section", sources.root.includes("showcase_selection_section.zui#ShowcaseSelectionSection")],
  ["root imports collections section", sources.root.includes("showcase_collections_section.zui#ShowcaseCollectionsSection")],
  ["label atom showcased", allNeedles(showcase, 'component = "Label"', 'control_id = "LabelDemo"')],
  ["icon atom showcased", allNeedles(showcase, 'control_id = "IconDemo"', 'control_id = "SvgIconDemo"')],
  ["button state matrix showcased", allNeedles(
    sources.input,
    'control_id = "ButtonDemo"',
    'control_id = "ButtonOutlinedDemo"',
    'control_id = "ButtonTextDemo"',
    'control_id = "ButtonDangerDemo"',
    'control_id = "ButtonDisabledDemo"',
    'button_variant = "outlined"',
    'button_variant = "text"',
    'button_color = "error"',
    'hovered = true',
    'pressed = true',
    'focused = true',
    'disabled = true',
  )],
  ["icon button atom showcased", allNeedles(sources.input, 'component = "IconButton"', 'control_id = "IconButtonDemo"')],
  ["field atoms showcased", allNeedles(sources.input, 'control_id = "InputFieldDemo"', 'control_id = "TextFieldDemo"', 'control_id = "NumberFieldDemo"')],
  ["selection controls showcased", allNeedles(sources.input, 'control_id = "CheckboxDemo"', 'control_id = "RadioDemo"', 'control_id = "ToggleButtonDemo"')],
  ["slider atoms showcased", allNeedles(
    sources.input,
    'component = "Slider"',
    'control_id = "SliderDemo"',
    'value_percent = 0.42',
    'component = "RangeSlider"',
    'control_id = "RangeSliderDemo"',
    'range_min_percent = 0.28',
    'focused_thumb = "upper"',
  )],
  ["tabs atoms showcased", allNeedles(
    sources.input,
    'component = "Tab"',
    'control_id = "TabDemo"',
    'component = "Tabs"',
    'control_id = "TabStripDemo"',
    'selection_follows_focus = true',
    'disabled_options = ["console"]',
  )],
  ["segmented control showcased", allNeedles(sources.input, 'component = "SegmentedControl"', 'control_id = "SegmentedControlDemo"')],
  ["dropdown triggers showcased", allNeedles(sources.selection, 'control_id = "DropdownDemo"', 'control_id = "ComboBoxDemo"')],
  ["progress and badge showcased", allNeedles(sources.visual, 'control_id = "ProgressBarDemo"', 'control_id = "BadgeDemo"')],
  ["divider and skeleton showcased", allNeedles(sources.visual, 'control_id = "SeparatorDemo"', 'component = "Skeleton"', 'control_id = "SkeletonDemo"')],
  ["category mapping includes L1 additions", allNeedles(
    sources.categories,
    '"SkeletonDemo"',
    '"TabDemo"',
    '"TabStripDemo"',
    '"SliderDemo"',
    '"RangeSliderDemo"',
  )],
  ["default state maps L1 additions", allNeedles(
    sources.defaults,
    '"SkeletonDemo" => Some("Skeleton")',
    '"TabDemo" => Some("Tab")',
    '"TabStripDemo" => Some("Tabs")',
    '"SliderDemo" => Some("Slider")',
    '"RangeSliderDemo" => Some("RangeSlider")',
  )],
  ["all demo controls have category mapping", missingCategoryMappings.length === 0],
  ["all demo controls have default component mapping", missingComponentMappings.length === 0],
  ["all category mappings point at authored demo controls", staleCategoryMappings.length === 0],
  ["all default component mappings point at authored demo controls", staleComponentMappings.length === 0],
  ["all demo component mappings match authored zui component", mismatchedAuthoredComponents.length === 0],
  ["all demo categories match authored showcase section", mismatchedSectionCategories.length === 0],
  ["all authored showcase events have Rust bindings", missingRustBindings.length === 0],
  ["all Rust showcase bindings are authored in zui assets", extraRustBindings.length === 0],
  ["authored showcase event routes match Rust demo actions", mismatchedShowcaseEventRoutes.length === 0],
  ["authored showcase event routes use canonical namespace", nonCanonicalEventNamespaces.length === 0],
  ["showcase category navigation covers category constants and bindings", categoryNavigationIsComplete()],
  ["Rust showcase binding action kinds are handled by demo state", unsupportedShowcaseBindingActions.length === 0],
  ["typed showcase action bindings have deterministic demo inputs", missingTypedShowcaseActionInputs.length === 0],
  ["collection value-change bindings have deterministic value inputs", missingCollectionValueActionInputs.length === 0],
  ["drag-delta bindings have deterministic delta inputs", missingDragDeltaActionInputs.length === 0],
  ["Rust showcase bindings that require inputs have deterministic demo inputs", missingRequiredShowcaseActionInputs.length === 0],
  ["retained-host showcase action suffixes resolve to Rust bindings", missingRetainedHostActionBindings.length === 0],
  ["showcase demo-input action matchers resolve to authored bindings", staleShowcaseActionInputMatchers.length === 0],
  ["retained-host showcase test action ids resolve to Rust bindings", missingRetainedHostTestActionBindings.length === 0],
  ["Rust showcase tests reference authored binding ids", missingRustTestShowcaseBindings.length === 0],
  [
    "category-filter test references every authored demo control",
    missingCategoryFilterDemoControlReferences.length === 0,
  ],
  [
    "category-filter demo groups match Rust category mappings",
    missingCategoryFilterDemoGroups.length === 0 &&
      unknownCategoryFilterDemoGroups.length === 0 &&
      staleCategoryFilterDemoGroupReferences.length === 0 &&
      duplicateCategoryFilterDemoGroupReferences.length === 0 &&
      mismatchedCategoryFilterDemoGroupCategories.length === 0,
  ],
  [
    "category-filter aggregate helper covers every demo group",
    missingCategoryFilterAggregateDemoGroups.length === 0 &&
      unknownCategoryFilterAggregateDemoGroups.length === 0 &&
      duplicateCategoryFilterAggregateDemoGroups.length === 0,
  ],
  ["category-filter test covers every category nav binding", missingCategoryFilterSelectBindings.length === 0],
  [
    "category-filter assertions use mapped demo groups",
    missingCategoryFilterGroupAssertions.length === 0,
  ],
  [
    "category-filter test asserts nav selected state for every category",
    missingCategoryFilterProjectionBlocks.length === 0 &&
      missingCategoryFilterSelectedAssertions.length === 0 &&
      missingCategoryFilterDeselectedAssertions.length === 0 &&
      categoryFilterSelectedHelperComplete,
  ],
  ["projection roles include L1 additions", allNeedles(
    sources.viewProjection,
    '"RangeSlider" => "range-slider"',
    '"SegmentedControl" => "segmented-control"',
    '"Tab" => "tab"',
    '"Tabs" => "tabs"',
    '"Divider" | "Separator" => "divider"',
  )],
  ["retained-host projection asserts L1 additions", allNeedles(
    sources.retainedHostTest,
    'let slider = template_node(&nodes, "SliderDemo");',
    'assert_eq!(slider.component_role.as_str(), "slider");',
    '"ui_component_showcase.slider_drag_update"',
    '"ui_component_showcase.slider_changed"',
    'let range_slider = template_node(&nodes, "RangeSliderDemo");',
    'assert_eq!(range_slider.component_role.as_str(), "range-slider");',
    'assert_eq!(range_slider.layout_second_cell_offset_x, 28.0);',
    '"ui_component_showcase.range_slider_drag_update"',
    '"ui_component_showcase.range_slider_changed"',
    'let tab = template_node(&nodes, "TabDemo");',
    'assert_eq!(tab.component_role.as_str(), "tab");',
    '"ui_component_showcase.tab_changed"',
    'let tab_strip = template_node(&nodes, "TabStripDemo");',
    'assert_eq!(tab_strip.component_role.as_str(), "tabs");',
    'assert_eq!(tab_strip.structured_options.row_count(), 3);',
    '"ui_component_showcase.tab_strip_changed"',
    'let skeleton = template_node(&nodes, "SkeletonDemo");',
    'assert_eq!(skeleton.component_role.as_str(), "skeleton");',
  )],
  ["category filter test isolates L1 additions", allNeedles(
    sources.categoryTest,
    '"UiComponentShowcase/ShowInputCategory"',
    'node_by_control_id("TabDemo").is_some()',
    'node_by_control_id("TabStripDemo").is_some()',
    'node_by_control_id("SliderDemo").is_none()',
    '"UiComponentShowcase/ShowNumericCategory"',
    'node_by_control_id("SliderDemo").is_some()',
    'node_by_control_id("TabDemo").is_none()',
    '"UiComponentShowcase/ShowFeedbackCategory"',
    'node_by_control_id("SkeletonDemo").is_some()',
    '"UiComponentShowcase/ShowAllCategory"',
  ) && allPatterns(
    sources.categoryTest,
    /node_by_control_id\("ButtonDisabledDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("RangeSliderDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("RangeSliderDemo"\)\s*\.is_none\(\)/,
    /node_by_control_id\("SkeletonDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("SkeletonDemo"\)\s*\.is_none\(\)/,
    /node_by_control_id\("TableRowDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("TableRowDemo"\)\s*\.is_none\(\)/,
    /node_by_control_id\("VirtualListDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("VirtualListDemo"\)\s*\.is_none\(\)/,
    /node_by_control_id\("WorldSpaceSurfaceDemo"\)\s*\.is_some\(\)/,
    /node_by_control_id\("WorldSpaceSurfaceDemo"\)\s*\.is_none\(\)/,
  )],
];

const failed = checks.filter(([, passed]) => !passed);
for (const [name, passed] of checks) {
  console.log(`${passed ? "ok" : "fail"} ${name}`);
}

if (failed.length > 0) {
  const details = [
    missingCategoryMappings.length > 0
      ? `missing category mapping: ${missingCategoryMappings.join(", ")}`
      : "",
    missingComponentMappings.length > 0
      ? `missing default component mapping: ${missingComponentMappings.join(", ")}`
      : "",
    staleCategoryMappings.length > 0
      ? `stale category mapping for non-authored demo controls: ${staleCategoryMappings.join(", ")}`
      : "",
    staleComponentMappings.length > 0
      ? `stale default component mapping for non-authored demo controls: ${staleComponentMappings.join(", ")}`
      : "",
    mismatchedAuthoredComponents.length > 0
      ? `authored component mismatch: ${mismatchedAuthoredComponents
          .map(
            ({ controlId, component, section }) =>
              `${controlId}@${section} zui=${component} mapped=${componentMappingByControlId.get(controlId)}`,
          )
          .join(", ")}`
      : "",
    mismatchedSectionCategories.length > 0
      ? `section category mismatch: ${mismatchedSectionCategories
          .map(
            ({ controlId, section }) =>
              `${controlId}@${section} mapped=${categoryMappingByControlId.get(controlId)}`,
          )
          .join(", ")}`
      : "",
    missingRustBindings.length > 0
      ? `missing Rust binding: ${missingRustBindings
          .map(({ id, controlId }) => `${id}@${controlId}`)
          .join(", ")}`
      : "",
    extraRustBindings.length > 0
      ? `Rust binding not authored in zui assets: ${extraRustBindings
          .map(({ id, controlId }) => `${id}@${controlId}`)
          .join(", ")}`
      : "",
    mismatchedRustBindings.length > 0
      ? `Rust binding mismatch: ${mismatchedRustBindings
          .map(({ id, controlId, event }) => {
            const binding = rustBindingById.get(id);
            return `${id} zui=${controlId}/${event} rust=${binding.controlId}/${binding.event}`;
          })
          .join(", ")}`
      : "",
    mismatchedShowcaseEventRoutes.length > 0
      ? `showcase event route/action mismatch: ${mismatchedShowcaseEventRoutes
          .map(({ id, route }) => {
            const binding = rustBindingById.get(id);
            return `${id} zui=${route} rust=${showcaseRouteForAction(binding.action)}`;
          })
          .join(", ")}`
      : "",
    nonCanonicalEventNamespaces.length > 0
      ? `non-canonical event namespace: ${nonCanonicalEventNamespaces
          .map(({ id, route }) => `${id}->${route}`)
          .join(", ")}`
      : "",
    !categoryNavigationIsComplete() ? categoryNavigationDetails() : "",
    unsupportedShowcaseBindingActions.length > 0
      ? `unsupported showcase binding action kind: ${unsupportedShowcaseBindingActions
          .map(({ id, action }) => `${id}->${actionKind(action)} (${action})`)
          .join(", ")}`
      : "",
    missingTypedShowcaseActionInputs.length > 0
      ? `missing typed showcase action input: ${missingTypedShowcaseActionInputs
          .map(({ id, action }) => `${id}->${action}`)
          .join(", ")}`
      : "",
    missingCollectionValueActionInputs.length > 0
      ? `missing collection value showcase action input: ${missingCollectionValueActionInputs
          .map(({ id, action }) => `${id}->${action}`)
          .join(", ")}`
      : "",
    missingDragDeltaActionInputs.length > 0
      ? `missing drag delta showcase action input: ${missingDragDeltaActionInputs
          .map(({ id, action }) => `${id}->${action}`)
          .join(", ")}`
      : "",
    missingRequiredShowcaseActionInputs.length > 0
      ? `missing required showcase action input: ${missingRequiredShowcaseActionInputs
          .map(({ id, action }) => `${id}->${action}`)
          .join(", ")}`
      : "",
    missingRetainedHostActionBindings.length > 0
      ? `retained-host action suffix missing Rust binding: ${missingRetainedHostActionBindings.join(", ")}`
      : "",
    staleShowcaseActionInputMatchers.length > 0
      ? `stale showcase action input matcher: ${staleShowcaseActionInputMatchers
          .map(({ matcher, source }) => `${matcher} (${source})`)
          .join(", ")}`
      : "",
    missingRetainedHostTestActionBindings.length > 0
      ? `retained-host test action id missing Rust binding: ${missingRetainedHostTestActionBindings.join(", ")}`
      : "",
    missingRustTestShowcaseBindings.length > 0
      ? `Rust showcase test binding id missing Rust binding: ${missingRustTestShowcaseBindings.join(", ")}`
      : "",
    missingCategoryFilterDemoControlReferences.length > 0
      ? `category-filter test missing authored demo controls: ${missingCategoryFilterDemoControlReferences.join(", ")}`
      : "",
    missingCategoryFilterDemoGroups.length > 0
      ? `category-filter test missing demo group constants: ${missingCategoryFilterDemoGroups.join(", ")}`
      : "",
    unknownCategoryFilterDemoGroups.length > 0
      ? `category-filter test has unknown demo group constants: ${unknownCategoryFilterDemoGroups.join(", ")}`
      : "",
    missingCategoryFilterAggregateDemoGroups.length > 0
      ? `category-filter aggregate helper missing demo groups: ${missingCategoryFilterAggregateDemoGroups.join(", ")}`
      : "",
    unknownCategoryFilterAggregateDemoGroups.length > 0
      ? `category-filter aggregate helper has unknown demo groups: ${unknownCategoryFilterAggregateDemoGroups.join(", ")}`
      : "",
    duplicateCategoryFilterAggregateDemoGroups.length > 0
      ? `category-filter aggregate helper duplicates demo groups: ${duplicateCategoryFilterAggregateDemoGroups.join(", ")}`
      : "",
    staleCategoryFilterDemoGroupReferences.length > 0
      ? `category-filter demo groups reference non-authored controls: ${staleCategoryFilterDemoGroupReferences
          .map(({ groupName, controlId }) => `${groupName}->${controlId}`)
          .join(", ")}`
      : "",
    duplicateCategoryFilterDemoGroupReferences.length > 0
      ? `category-filter demo groups duplicate controls: ${duplicateCategoryFilterDemoGroupReferences
          .map(({ controlId, groups }) => `${controlId}@${groups.join("+")}`)
          .join(", ")}`
      : "",
    mismatchedCategoryFilterDemoGroupCategories.length > 0
      ? `category-filter demo group/category mismatch: ${mismatchedCategoryFilterDemoGroupCategories
          .map(
            ({ groupName, controlId, expectedCategory, actualCategory }) =>
              `${groupName}->${controlId} expected=${expectedCategory} actual=${actualCategory}`,
          )
          .join(", ")}`
      : "",
    missingCategoryFilterSelectBindings.length > 0
      ? `category-filter test missing SelectCategory binding coverage: ${missingCategoryFilterSelectBindings.join(", ")}`
      : "",
    missingCategoryFilterGroupAssertions.length > 0
      ? `category-filter test missing mapped demo group assertion: ${missingCategoryFilterGroupAssertions
          .map(({ id, action }) => `${id}->${action}`)
          .join(", ")}`
      : "",
    missingCategoryFilterProjectionBlocks.length > 0
      ? `category-filter test missing projection block after SelectCategory binding: ${missingCategoryFilterProjectionBlocks
          .map(({ id }) => id)
          .join(", ")}`
      : "",
    missingCategoryFilterSelectedAssertions.length > 0
      ? `category-filter test missing selected=true nav assertion: ${missingCategoryFilterSelectedAssertions
          .map(({ id, controlId }) => `${id}@${controlId}`)
          .join(", ")}`
      : "",
    missingCategoryFilterDeselectedAssertions.length > 0
      ? `category-filter test missing selected=false nav assertion: ${missingCategoryFilterDeselectedAssertions
          .map(({ id, controlId, missingControlId }) => `${id}@${controlId}->${missingControlId}`)
          .join(", ")}`
      : "",
    !categoryFilterSelectedHelperComplete
      ? "category-filter selected-state helper is missing or does not cover every Show*Category nav control"
      : "",
  ]
    .filter(Boolean)
    .join("; ");
  console.error(
    `Showcase state matrix failed: ${failed.map(([name]) => name).join(", ")}${
      details ? `; ${details}` : ""
    }`,
  );
  process.exit(1);
}

console.log(
  `showcase state matrix: l1Atoms=16 demoControls=${demoControlIds.length} events=${authoredEvents.length} sections=4 checks=${checks.length}`,
);
console.log("ok showcase L1 state matrix contract");

function allNeedles(source, ...needles) {
  return needles.every((needle) => source.includes(needle));
}

function allPatterns(source, ...patterns) {
  return patterns.every((pattern) => pattern.test(source));
}

function readDemoNodes(source, section) {
  return source
    .split(/\n(?=\[nodes\.)/g)
    .flatMap((block) => {
      const controlId = block.match(/control_id\s*=\s*"([^"]+)"/)?.[1];
      const component = block.match(/component\s*=\s*"([^"]+)"/)?.[1];
      if (!controlId?.endsWith("Demo") || !component) {
        return [];
      }
      return [{ section, controlId, component }];
    });
}

function readShowcaseEvents(source, asset) {
  return source
    .split(/\n(?=\[nodes\.)/g)
    .flatMap((block) => {
      const controlId = block.match(/control_id\s*=\s*"([^"]+)"/)?.[1];
      if (!controlId) {
        return [];
      }
      return [...block.matchAll(
        /\{\s*id\s*=\s*"([^"]+)",\s*event\s*=\s*"([^"]+)",\s*route\s*=\s*"([^"]+)"\s*\}/g,
      )].map(([, id, event, route]) => ({ asset, controlId, id, event, route }));
    });
}

function showcaseBindingsFromRust(source) {
  return [...source.matchAll(
    /showcase_binding_entry\(\s*"([^"]+)",\s*"([^"]+)",\s*EditorUiEventKind::([A-Za-z]+),\s*"([^"]+)"/g,
  )].map(([, id, controlId, event, action]) => ({ id, controlId, event, action }));
}

function categoryConstantsFromRust(source) {
  return [...source.matchAll(/const\s+(CATEGORY_[A-Z_]+):\s*&str\s*=\s*"([^"]+)"/g)].map(
    ([, name, value]) => ({ name, value }),
  );
}

function navCategoryMappingsFromRust(source, constants) {
  const constantValues = new Map(constants.map(({ name, value }) => [name, value]));
  const navBlock = sourceBetween(source, "fn nav_category_for_control", "fn demo_category_for_control");
  return new Map(
    [...navBlock.matchAll(/"([^"]+)"\s*=>\s*Some\((CATEGORY_[A-Z_]+)\)/g)]
      .map(([, controlId, constant]) => [controlId, constantValues.get(constant) ?? constant]),
  );
}

function categoryNavControlIdsFromAsset(source) {
  return new Set(
    [...source.matchAll(/control_id\s*=\s*"(Show[A-Za-z]+Category)"/g)].map(([, controlId]) => controlId),
  );
}

function categoryNavigationIsComplete() {
  return (
    missingNavCategories.length === 0 &&
    unknownNavCategories.length === 0 &&
    missingCategoryNavNodes.length === 0 &&
    unmappedCategoryNavNodes.length === 0 &&
    missingSelectCategoryBindings.length === 0 &&
    staleSelectCategoryBindings.length === 0 &&
    mismatchedSelectCategoryBindings.length === 0
  );
}

function categoryNavigationDetails() {
  return [
    missingNavCategories.length > 0
      ? `missing category nav for constants: ${missingNavCategories.join(", ")}`
      : "",
    unknownNavCategories.length > 0
      ? `category nav uses unknown categories: ${unknownNavCategories.join(", ")}`
      : "",
    missingCategoryNavNodes.length > 0
      ? `category nav Rust mapping missing zui node: ${missingCategoryNavNodes.join(", ")}`
      : "",
    unmappedCategoryNavNodes.length > 0
      ? `category nav zui node missing Rust mapping: ${unmappedCategoryNavNodes.join(", ")}`
      : "",
    missingSelectCategoryBindings.length > 0
      ? `category nav missing SelectCategory binding: ${missingSelectCategoryBindings
          .map(([controlId, category]) => `${controlId}->${category}`)
          .join(", ")}`
      : "",
    staleSelectCategoryBindings.length > 0
      ? `SelectCategory binding missing nav mapping: ${staleSelectCategoryBindings
          .map(({ controlId, action }) => `${controlId}->${action}`)
          .join(", ")}`
      : "",
    mismatchedSelectCategoryBindings.length > 0
      ? `SelectCategory binding/nav mismatch: ${mismatchedSelectCategoryBindings
          .map(({ controlId, action }) => `${controlId} nav=${navCategoryByControlId.get(controlId)} action=${action}`)
          .join(", ")}`
      : "",
  ].filter(Boolean).join("; ");
}

function retainedHostActionSuffixesFromRust(source) {
  const suffixes = new Set();
  for (const [, suffix] of source.matchAll(/Some\("([A-Za-z0-9]+)"\)/g)) {
    suffixes.add(suffix);
  }
  for (const [, suffix] of source.matchAll(
    /\("(?:Find|Open|Clear|Add|Set|Remove|Move)",\s*"([A-Za-z0-9]+)"\)/g,
  )) {
    suffixes.add(suffix);
  }
  return [...suffixes].sort();
}

function showcaseRouteForAction(action) {
  return `component_lab.showcase.${action.split(".").map(camelToSnake).join(".")}`;
}

function showcaseActionIdForBindingId(bindingId) {
  return `ui_component_showcase.${camelToSnake(showcaseBindingSuffix(bindingId))}`;
}

function showcaseActionIdsFromSource(source) {
  return [
    ...new Set(
      [...source.matchAll(/"((?:ui_component_showcase)\.[^"]+)"/g)]
        .map(([, actionId]) => actionId)
        .filter((actionId) => !actionId.includes("{")),
    ),
  ].sort();
}

function showcaseBindingIdsFromSources(sources) {
  return [
    ...new Set(
      sources.flatMap((source) =>
        [...source.matchAll(/"(UiComponentShowcase\/[^"]+)"/g)].map(([, bindingId]) => bindingId),
      ),
    ),
  ].sort();
}

function demoControlIdsFromSource(source) {
  return [
    ...new Set([...source.matchAll(/"([A-Za-z0-9]+Demo)"/g)].map(([, controlId]) => controlId)),
  ].sort();
}

function categoryFilterDemoGroupsFromRust(source) {
  const groups = new Map();
  for (const [, groupName, body] of source.matchAll(
    /const\s+(SHOWCASE_[A-Z_]+_DEMO_CONTROL_IDS):\s*&\[&str\]\s*=\s*&\[(.*?)\];/gs,
  )) {
    groups.set(
      groupName,
      [...new Set([...body.matchAll(/"([A-Za-z0-9]+Demo)"/g)].map(([, controlId]) => controlId))]
        .sort(),
    );
  }
  return groups;
}

function categoryFilterAggregateDemoGroupsFromRust(source) {
  const body = source.match(
    /const\s+SHOWCASE_DEMO_CONTROL_GROUPS:\s*&\[&\[&str\]\]\s*=\s*&\[(.*?)\];/s,
  )?.[1] ?? "";
  return [...body.matchAll(/\b(SHOWCASE_[A-Z_]+_DEMO_CONTROL_IDS)\b/g)]
    .map(([, groupName]) => groupName);
}

function duplicateCategoryFilterDemoGroupReferencesFromGroups(groups) {
  const firstGroupByControlId = new Map();
  const duplicates = [];
  for (const [groupName, controlIds] of groups.entries()) {
    for (const controlId of controlIds) {
      const firstGroup = firstGroupByControlId.get(controlId);
      if (firstGroup) {
        duplicates.push({ controlId, groups: [firstGroup, groupName] });
      } else {
        firstGroupByControlId.set(controlId, groupName);
      }
    }
  }
  return duplicates;
}

function duplicateValues(values) {
  const seen = new Set();
  const duplicates = new Set();
  for (const value of values) {
    if (seen.has(value)) {
      duplicates.add(value);
    } else {
      seen.add(value);
    }
  }
  return [...duplicates].sort();
}

function categoryFilterTestBlock(source, bindingId) {
  const bindingIndex = source.indexOf(`"${bindingId}"`);
  if (bindingIndex === -1) {
    return "";
  }
  const projectionIndex = source.indexOf(
    "let host_projection = project_showcase(&runtime);",
    bindingIndex,
  );
  const blockStart = projectionIndex === -1 ? bindingIndex : projectionIndex;
  const nextBindingIndex = source.indexOf("apply_showcase_binding(", blockStart + 1);
  return source.slice(blockStart, nextBindingIndex === -1 ? source.length : nextBindingIndex);
}

function hasCategorySelectedAssertion(source, controlId, selected) {
  const selectedByHelper = categorySelectedHelperControlId(source);
  if (selectedByHelper) {
    return selected ? selectedByHelper === controlId : selectedByHelper !== controlId;
  }
  return new RegExp(
    `node_by_control_id\\("${escapeRegExp(controlId)}"\\)[\\s\\S]{0,360}?properties\\.get\\("selected"\\)[\\s\\S]{0,180}?Some\\(&RetainedUiHostValue::Bool\\(${selected}\\)\\)`,
  ).test(source);
}

function categorySelectedHelperControlId(source) {
  return source.match(/assert_selected_category\(&host_projection,\s*"([^"]+)"\)/)?.[1] ?? "";
}

function categorySelectedHelperCoversNavControls(source, controlIds) {
  const helperStart = source.indexOf("const SHOWCASE_CATEGORY_CONTROL_IDS");
  const testStart = source.indexOf("#[test]", helperStart);
  const helperBlock =
    helperStart === -1 || testStart === -1 ? "" : source.slice(helperStart, testStart);
  return (
    helperBlock.includes("SHOWCASE_CATEGORY_CONTROL_IDS") &&
    /Bool\(\s*\*control_id\s*==\s*selected_control_id\s*\)/.test(helperBlock) &&
    [...controlIds].every((controlId) => helperBlock.includes(`"${controlId}"`))
  );
}

function showcaseActionKindsFromRuntime(source) {
  const kinds = new Set();
  if (source.includes('strip_prefix("SelectCategory.")')) {
    kinds.add("SelectCategory");
  }
  const actionBlock = sourceBetween(
    source,
    "fn component_event_for_action",
    "fn context_action_menu_option_id",
  );
  for (const [, kind] of actionBlock.matchAll(/"([A-Za-z]+)"\s*(?:=>|\|)/g)) {
    kinds.add(kind);
  }
  return kinds;
}

function staleShowcaseActionInputMatchersFromRust(source, bindings) {
  const bindingActionNeedles = bindings.map(({ id }) => camelToSnake(showcaseBindingSuffix(id)));
  return showcaseActionInputMatchersFromRust(source)
    .filter(
      ({ matcher }) => !bindingActionNeedles.some((bindingNeedle) => bindingNeedle.includes(matcher)),
    )
    .sort((left, right) => left.matcher.localeCompare(right.matcher));
}

function showcaseActionInputMatchersFromRust(source) {
  const constants = rustStringConstants(source);
  const matchers = new Map();
  for (const [, matcher] of source.matchAll(/action_matches\(action(?:_id)?,\s*"([^"]+)"\)/g)) {
    matchers.set(`direct:${matcher}`, { matcher, source: "action_matches" });
  }
  for (const [, constantName] of source.matchAll(
    /action_matches_binding_suffix\(action(?:_id)?,\s*([A-Z0-9_]+)\)/g,
  )) {
    const suffix = constants.get(constantName);
    if (suffix) {
      matchers.set(`suffix:${constantName}`, {
        matcher: camelToSnake(suffix),
        source: constantName,
      });
    }
  }
  return [...matchers.values()];
}

function rustStringConstants(source) {
  return new Map(
    [...source.matchAll(/const\s+([A-Z0-9_]+):\s*&str\s*=\s*"([^"]+)"/g)].map(
      ([, name, value]) => [name, value],
    ),
  );
}

function hasShowcaseActionInputMapping(source, bindingSuffix, inputVariant) {
  const actionNeedle = camelToSnake(bindingSuffix);
  return new RegExp(
    `action_matches\\(action,\\s*"${escapeRegExp(actionNeedle)}"\\)[\\s\\S]{0,240}?UiComponentShowcaseDemoEventInput::${escapeRegExp(inputVariant)}\\b`,
  ).test(source);
}

function hasShowcaseActionInputChoice(source, bindingSuffix, inputChoices) {
  return inputChoices.some((inputChoice) =>
    hasShowcaseActionInputPattern(source, bindingSuffix, inputChoice),
  );
}

function hasShowcaseActionInputPattern(source, bindingSuffix, inputChoice) {
  const actionNeedle = camelToSnake(bindingSuffix);
  const pattern =
    inputChoice.kind === "helper"
      ? `${escapeRegExp(inputChoice.name)}\\s*\\(`
      : `UiComponentShowcaseDemoEventInput::${escapeRegExp(inputChoice.name)}\\b`;
  return showcaseActionInputMatcherWindowsFromRust(source).some(
    ({ matcher, window }) => actionNeedle.includes(matcher) && new RegExp(pattern).test(window),
  );
}

function showcaseActionInputMatcherWindowsFromRust(source) {
  const constants = rustStringConstants(source);
  const windows = [];
  for (const match of source.matchAll(/action_matches\(action(?:_id)?,\s*"([^"]+)"\)/g)) {
    windows.push({ matcher: match[1], window: source.slice(match.index, match.index + 520) });
  }
  for (const match of source.matchAll(
    /action_matches_binding_suffix\(action(?:_id)?,\s*([A-Z0-9_]+)\)/g,
  )) {
    const suffix = constants.get(match[1]);
    if (suffix) {
      windows.push({
        matcher: camelToSnake(suffix),
        window: source.slice(match.index, match.index + 520),
      });
    }
  }
  return windows;
}

function variantInput(name) {
  return { kind: "variant", name };
}

function helperInput(name) {
  return { kind: "helper", name };
}

function showcaseBindingSuffix(bindingId) {
  return bindingId.replace(/^UiComponentShowcase\//, "");
}

function actionKind(action) {
  return action.split(".")[0];
}

function camelToSnake(value) {
  return value
    .replace(/[^A-Za-z0-9]+/g, "_")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/_+/g, "_")
    .replace(/^_|_$/g, "")
    .toLowerCase();
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function componentMappingsFromRust(source) {
  const mappings = new Map();
  const arms = source.matchAll(/((?:"[^"]+"\s*(?:\|\s*)?)+)\s*=>\s*Some\("([^"]+)"\)/g);
  for (const [, controlIds, componentId] of arms) {
    for (const [, controlId] of controlIds.matchAll(/"([^"]+)"/g)) {
      mappings.set(controlId, componentId);
    }
  }
  return mappings;
}

function categoryMappingsFromRust(source) {
  const mappings = new Map();
  const arms = source.matchAll(
    /((?:"[^"]+"\s*(?:\|\s*)?)+)\s*=>\s*(?:\{\s*)?Some\((CATEGORY_[A-Z_]+)\)/g,
  );
  for (const [, controlIds, category] of arms) {
    for (const [, controlId] of controlIds.matchAll(/"([^"]+)"/g)) {
      mappings.set(controlId, category);
    }
  }
  return mappings;
}

function sourceBetween(source, startNeedle, endNeedle) {
  const start = source.indexOf(startNeedle);
  const end = source.indexOf(endNeedle, start + startNeedle.length);
  if (start === -1 || end === -1) {
    return "";
  }
  return source.slice(start, end);
}

function sourceFrom(source, startNeedle) {
  const start = source.indexOf(startNeedle);
  return start === -1 ? "" : source.slice(start);
}

function readRepo(path) {
  return readFileSync(new URL(path, repoRoot), "utf8");
}

function readRepoMany(paths) {
  return paths.map(readRepo).join("\n");
}
