import { isAbsolute, win32 } from 'node:path';
import { canonicalSha256 } from './zui-layout-review-contract';

export interface RuntimeSourceFingerprint {
  sourcePath: string;
  sha256: string;
}

export interface WorkbenchLocaleAudit {
  complete: boolean;
  actualLocale: string;
  expectedLocale: string;
  source: string;
}

const BROWSER_WORKBENCH_LOCALE_SOURCE =
  'penpot-prepared-workbench-presentation';
const NATIVE_WORKBENCH_LOCALE_SOURCE = 'editor-i18n-service';

function expectedWorkbenchLocale(locale: unknown): 'en' | 'zh-CN' | null {
  if (locale === 'en-US' || locale === 'en') return 'en';
  if (locale === 'zh-CN') return 'zh-CN';
  return null;
}

/** Record the locale on the exact product projection consumed by Penpot. */
export function capturePreparedWorkbenchLocaleAudit(
  preparedPresentation: unknown,
  caseLocale: unknown,
): WorkbenchLocaleAudit {
  const expectedLocale = expectedWorkbenchLocale(caseLocale);
  const actualLocale = isRecord(preparedPresentation)
    ? preparedPresentation['activeLocale']
    : undefined;
  const complete =
    expectedLocale !== null &&
    (actualLocale === 'en' || actualLocale === 'zh-CN') &&
    actualLocale === expectedLocale;
  return {
    complete,
    actualLocale: typeof actualLocale === 'string' ? actualLocale : '',
    expectedLocale: expectedLocale ?? '',
    source: BROWSER_WORKBENCH_LOCALE_SOURCE,
  };
}

/** Require independent renderer locale receipts to agree with the case. */
export function workbenchLocaleEvidenceErrors(
  penpotGeometry: unknown,
  engineGeometry: unknown,
  caseLocale: unknown,
): string[] {
  const expectedLocale = expectedWorkbenchLocale(caseLocale);
  if (!expectedLocale)
    return ['workbench case locale has no supported active-locale mapping'];
  const readAudit = (
    value: unknown,
    renderer: 'penpot' | 'engine',
    expectedSource: string,
  ): WorkbenchLocaleAudit | null => {
    if (!isRecord(value)) {
      return null;
    }
    const raw = value['localeAudit'];
    if (
      !isRecord(raw) ||
      !exactKeys(raw, ['complete', 'actualLocale', 'expectedLocale', 'source'])
    ) {
      errors.push(`${renderer}: missing or unsupported workbench locale audit`);
      return null;
    }
    if (raw['source'] !== expectedSource)
      errors.push(`${renderer}: workbench locale audit has an unsupported source`);
    if (raw['complete'] !== true)
      errors.push(`${renderer}: workbench locale audit is incomplete`);
    if (
      (raw['actualLocale'] !== 'en' && raw['actualLocale'] !== 'zh-CN') ||
      raw['expectedLocale'] !== expectedLocale
    )
      errors.push(`${renderer}: workbench locale receipt does not match the case`);
    return raw as unknown as WorkbenchLocaleAudit;
  };
  const errors: string[] = [];
  const penpot = readAudit(
    penpotGeometry,
    'penpot',
    BROWSER_WORKBENCH_LOCALE_SOURCE,
  );
  const engine = readAudit(
    engineGeometry,
    'engine',
    NATIVE_WORKBENCH_LOCALE_SOURCE,
  );
  for (const [renderer, audit] of [
    ['penpot', penpot],
    ['engine', engine],
  ] as const) {
    if (audit && audit.actualLocale !== expectedLocale)
      errors.push(`${renderer}: active locale differs from the case expectation`);
  }
  if (
    penpot &&
    engine &&
    penpot.actualLocale !== engine.actualLocale
  )
    errors.push('Active workbench locale differs between renderers');
  return errors;
}

interface TokenReceipt {
  complete: boolean;
  sha256: string;
  tokens: Record<string, unknown>;
}

interface RuntimeLoadedSourceFile {
  assetId: string;
  sourcePath: string;
  resourceUri: string;
  physicalPath: string;
  sha256: string;
  catalogMatches: boolean;
  currentFileMatches: boolean;
}

interface RuntimeLoadedSources {
  complete: boolean;
  documentIds: string[];
  files: RuntimeLoadedSourceFile[];
  unresolvedImports: unknown[];
}

const ACTIVE_DESIGN_TOKEN_ROOT_KEYS = [
  'chrome',
  'controls',
  'density',
  'id',
  'palette',
  'state_roles',
  'typography',
] as const;
const PALETTE_FIELDS = [
  'accent',
  'accent_soft',
  'border',
  'border_disabled',
  'error',
  'error_container',
  'focus_ring',
  'info',
  'info_container',
  'popup',
  'separator_soft',
  'separator_strong',
  'shadow',
  'success',
  'success_container',
  'surface',
  'surface_disabled',
  'surface_hover',
  'surface_recessed',
  'surface_selected',
  'text_disabled',
  'text_primary',
  'text_secondary',
  'track',
  'warning',
  'warning_container',
] as const;
const TYPOGRAPHY_NUMERIC_FIELDS = [
  'body_size',
  'caption_size',
  'overlay_size',
  'heading_size',
  'title_size',
  'line_height',
] as const;
const TYPOGRAPHY_WEIGHT_FIELDS = [
  'body_weight',
  'medium_weight',
  'strong_weight',
  'emphasis_weight',
  'code_weight',
] as const;
const TYPOGRAPHY_STRING_FIELDS = [
  'ui_family',
  'ui_strong_family',
  'code_family',
] as const;
const CONTROL_FIELDS = [
  'large_height',
  'default_height',
  'compact_height',
  'dense_height',
  'small_radius',
  'control_radius',
  'large_radius',
  'panel_radius',
  'pill_radius',
  'border_width',
] as const;
const DENSITY_FIELDS = [
  'gap_xsmall',
  'gap_tight',
  'gap_small',
  'gap_regular',
  'gap_medium',
  'gap_large',
  'gap_group',
  'drawer_padding',
  'panel_padding',
  'toolbar_action_width',
  'toolbar_wide_action_width',
  'ui_asset_action_min_width',
  'ui_asset_action_preferred_width',
  'ui_asset_action_max_width',
  'ui_asset_side_min_width',
  'ui_asset_side_preferred_width',
  'ui_asset_center_min_width',
  'ui_asset_center_preferred_width',
  'ui_asset_header_kind_min_width',
  'ui_asset_header_kind_preferred_width',
  'ui_asset_header_kind_max_width',
  'ui_asset_tool_min_width',
  'ui_asset_tool_preferred_width',
  'ui_asset_tool_max_width',
  'command_palette_min_width',
  'command_palette_preferred_width',
  'command_palette_max_width',
  'command_palette_min_height',
  'command_palette_preferred_height',
  'command_palette_max_height',
  'dialog_min_width',
  'dialog_preferred_width',
  'dialog_max_width',
  'dialog_min_height',
  'dialog_preferred_height',
  'dialog_max_height',
  'confirm_dialog_preferred_width',
  'confirm_dialog_max_width',
  'confirm_dialog_min_height',
  'confirm_dialog_preferred_height',
  'confirm_dialog_max_height',
  'notification_panel_min_width',
  'notification_panel_preferred_width',
  'notification_panel_max_width',
  'notification_panel_min_height',
  'notification_panel_preferred_height',
  'notification_panel_max_height',
  'caption_min_height',
  'caption_preferred_height',
  'caption_max_height',
  'label_min_height',
  'label_preferred_height',
  'label_max_height',
  'chip_min_width',
  'chip_preferred_width',
  'chip_max_width',
  'axis_value_field_min_width',
  'axis_value_field_preferred_width',
  'axis_value_field_max_width',
  'row_height',
  'left_drawer_width',
  'right_drawer_width',
  'bottom_output_height',
  'breakpoint_ultra_width',
  'breakpoint_narrow_width',
  'breakpoint_wide_width',
  'compact_side_width',
  'ultra_compact_side_width',
  'compact_left_drawer_max_width',
  'compact_right_drawer_max_width',
  'compact_side_min_width',
  'minimum_document_width_fraction',
  'ultra_compact_left_drawer_max_width',
  'ultra_compact_right_drawer_max_width',
  'compact_bottom_available_height',
  'compact_bottom_max_height',
  'compact_bottom_max_available_fraction',
  'compact_bottom_min_height',
  'ultra_compact_bottom_available_height',
  'ultra_compact_bottom_max_height',
  'ultra_compact_bottom_max_available_fraction',
  'ultra_compact_bottom_min_height',
  'minimum_window_width',
  'minimum_window_height',
  'ultra_minimum_window_width',
  'ultra_minimum_window_height',
] as const;
const CHROME_FIELDS = [
  'top_bar_height',
  'host_bar_height',
  'workbench_toolbar_height',
  'workbench_toolbar_command_row_height',
  'workbench_toolbar_popup_command_offset_y',
  'workbench_toolbar_popup_module_offset_y',
  'status_bar_height',
  'panel_header_height',
  'document_header_height',
  'viewport_toolbar_height',
  'activity_rail_width',
  'separator_thickness',
  'splitter_hit_size',
] as const;
const STATE_ROLE_FIELDS = [
  'default',
  'hovered',
  'pressed',
  'selected',
  'focused',
  'disabled',
  'loading',
] as const;
const STATE_ROLE_VALUES = new Set([
  'surface0',
  'surface1',
  'surface2',
  'surface3',
  'surface_selected',
  'accent',
  'focus_ring',
  'border',
  'text_primary',
  'text_secondary',
  'text_disabled',
]);
const RUNTIME_SOURCE_FIELDS = [
  'assetId',
  'catalogMatches',
  'currentFileMatches',
  'physicalPath',
  'resourceUri',
  'sha256',
  'sourcePath',
] as const;

const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const exactKeys = (value: Record<string, unknown>, expected: readonly string[]) =>
  Object.keys(value).sort().join('\0') === [...expected].sort().join('\0');
const finiteNumber = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value);
const nonemptyString = (value: unknown): value is string =>
  typeof value === 'string' && value.trim().length > 0;
const validHash = (value: unknown): value is string =>
  typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);

function validTokenReceipt(value: unknown): TokenReceipt | null {
  if (!isRecord(value) || !exactKeys(value, ['complete', 'sha256', 'tokens']))
    return null;
  if (
    value['complete'] !== true ||
    !validHash(value['sha256']) ||
    !isRecord(value['tokens'])
  )
    return null;
  const tokens = value['tokens'];
  const keys = Object.keys(tokens);
  if (
    !keys.length ||
    keys.some((key) => !/^editor(?:\.[a-z0-9_-]+)+$/.test(key)) ||
    keys.some((key) => {
      const token = tokens[key];
      return !(
        (typeof token === 'string' && token.length > 0) ||
        finiteNumber(token)
      );
    })
  )
    return null;
  const receipt = value as unknown as TokenReceipt;
  if (canonicalSha256(tokens) !== receipt.sha256) return null;
  return receipt;
}

/** Build the browser receipt from the exact prepared ZUI token object. */
export function capturePreparedEditorTokenReceipt(
  preparedTokens: unknown,
): { receipt: TokenReceipt; errors: string[] } {
  const errors: string[] = [];
  const tokens: Record<string, unknown> = {};
  if (!isRecord(preparedTokens)) {
    errors.push('prepared editor token map is unavailable');
  } else {
    for (const [key, value] of Object.entries(preparedTokens)) {
      if (!key.startsWith('editor.')) continue;
      if (
        !/^editor(?:\.[a-z0-9_-]+)+$/.test(key) ||
        !(
          (typeof value === 'string' && value.length > 0) ||
          finiteNumber(value)
        )
      ) {
        errors.push(`prepared editor token is malformed: ${key}`);
        continue;
      }
      tokens[key] = value;
    }
  }
  if (!Object.keys(tokens).length)
    errors.push('prepared editor token map contains no canonical editor.* values');

  for (const [key, value] of Object.entries(tokens)) {
    if (!key.startsWith('editor.state.') || typeof value !== 'string') continue;
    const target = value.startsWith('$') ? value.slice(1) :
      value.startsWith('editor.') ? value :
        `editor.${value.replaceAll('_', '.')}`;
    if (!/^editor(?:\.[a-z0-9_-]+)+$/.test(target) || !Object.hasOwn(tokens, target)) {
      errors.push(`state-role reference cannot be resolved: ${key}`);
      continue;
    }
    tokens[key] = `$${target}`;
  }
  const receipt: TokenReceipt = {
    complete: errors.length === 0,
    sha256: canonicalSha256(tokens),
    tokens,
  };
  return { receipt, errors };
}

function activeDesignTokenSnapshotErrors(value: unknown): string[] {
  const errors: string[] = [];
  if (!isRecord(value) || !exactKeys(value, ['complete', 'sha256', 'tokens']))
    return ['engine: missing or unsupported active design-token receipt'];
  if (value['complete'] !== true)
    errors.push('engine: active design-token snapshot is incomplete');
  if (!validHash(value['sha256']) || !isRecord(value['tokens']))
    return ['engine: active design-token receipt is malformed'];
  const tokens = value['tokens'];
  if (canonicalSha256(tokens) !== value['sha256'])
    errors.push('engine: active design-token snapshot hash is stale');
  if (!exactKeys(tokens, ACTIVE_DESIGN_TOKEN_ROOT_KEYS)) {
    errors.push('engine: active design-token snapshot is incomplete');
    return errors;
  }
  if (tokens['id'] !== 'zircon.editor.workbench')
    errors.push('engine: active design-token snapshot has the wrong ID');

  const palette = tokens['palette'];
  if (!isRecord(palette) || !exactKeys(palette, PALETTE_FIELDS)) {
    errors.push('engine: active design-token palette is incomplete');
  } else {
    const colorFields = PALETTE_FIELDS.filter((field) => field !== 'surface');
    const validColor = (item: unknown): boolean =>
      isRecord(item) &&
      exactKeys(item, ['alpha', 'blue', 'green', 'red']) &&
      ['red', 'green', 'blue', 'alpha'].every((channel) => {
        const value = item[channel];
        return finiteNumber(value) && value >= 0 && value <= 1;
      });
    const surface = palette['surface'];
    if (
      !Array.isArray(surface) ||
      surface.length !== 4 ||
      !surface.every(validColor) ||
      colorFields.some((field) => !validColor(palette[field]))
    )
      errors.push('engine: active design-token palette has invalid colors');
  }

  const typography = tokens['typography'];
  const typographyFields = [
    ...TYPOGRAPHY_NUMERIC_FIELDS,
    ...TYPOGRAPHY_STRING_FIELDS,
    ...TYPOGRAPHY_WEIGHT_FIELDS,
    'font_smoothing',
    'utility_tab_text_role',
  ];
  if (!isRecord(typography) || !exactKeys(typography, typographyFields)) {
    errors.push('engine: active design-token typography is incomplete');
  } else if (
    TYPOGRAPHY_NUMERIC_FIELDS.some((field) => !finiteNumber(typography[field])) ||
    TYPOGRAPHY_WEIGHT_FIELDS.some(
      (field) =>
        !Number.isInteger(typography[field]) ||
        Number(typography[field]) < 0 ||
        Number(typography[field]) > 65535,
    ) ||
    TYPOGRAPHY_STRING_FIELDS.some((field) => !nonemptyString(typography[field])) ||
    !['grayscale', 'subpixel'].includes(String(typography['font_smoothing'])) ||
    !['ui', 'code'].includes(String(typography['utility_tab_text_role']))
  ) {
    errors.push('engine: active design-token typography has invalid values');
  }

  const controls = tokens['controls'];
  if (
    !isRecord(controls) ||
    !exactKeys(controls, CONTROL_FIELDS) ||
    CONTROL_FIELDS.some((field) => !finiteNumber(controls[field]))
  )
    errors.push('engine: active design-token controls are incomplete or invalid');

  for (const [section, fields] of [
    ['density', DENSITY_FIELDS],
    ['chrome', CHROME_FIELDS],
  ] as const) {
    const values = tokens[section];
    if (
      !isRecord(values) ||
      !exactKeys(values, fields) ||
      fields.some((field) => !finiteNumber(values[field]))
    )
      errors.push(`engine: active design-token ${section} is incomplete or invalid`);
  }

  const stateRoles = tokens['state_roles'];
  if (
    !isRecord(stateRoles) ||
    !exactKeys(stateRoles, STATE_ROLE_FIELDS) ||
    STATE_ROLE_FIELDS.some(
      (field) =>
        typeof stateRoles[field] !== 'string' ||
        !STATE_ROLE_VALUES.has(String(stateRoles[field])),
    )
  )
    errors.push('engine: active design-token state roles are incomplete or invalid');
  return errors;
}

/** Validate independently captured token receipts and compare the full inventory. */
export function runtimeTokenEvidenceErrors(
  penpotGeometry: unknown,
  engineGeometry: unknown,
): string[] {
  const errors: string[] = [];
  const penpot = isRecord(penpotGeometry) ? penpotGeometry : {};
  const engine = isRecord(engineGeometry) ? engineGeometry : {};
  const browserReceipt = validTokenReceipt(penpot['consumedTokens']);
  const nativeReceipt = validTokenReceipt(engine['consumedTokens']);
  if (!browserReceipt)
    errors.push('penpot: missing, incomplete, or stale consumed token receipt');
  if (!nativeReceipt)
    errors.push('engine: missing, incomplete, or stale consumed token receipt');
  errors.push(...activeDesignTokenSnapshotErrors(engine['activeDesignTokens']));
  if (!browserReceipt || !nativeReceipt) return errors;

  const browserKeys = Object.keys(browserReceipt.tokens).sort();
  const nativeKeys = Object.keys(nativeReceipt.tokens).sort();
  if (
    browserKeys.length !== nativeKeys.length ||
    browserKeys.some((key, index) => key !== nativeKeys[index])
  ) {
    errors.push('canonical editor token inventory differs between renderers');
  }
  for (const key of browserKeys) {
    if (!Object.hasOwn(nativeReceipt.tokens, key)) continue;
    const browserValue = browserReceipt.tokens[key];
    const nativeValue = nativeReceipt.tokens[key];
    if (typeof browserValue === 'number' && typeof nativeValue === 'number') {
      const tolerance = Math.max(
        1e-5,
        1e-5 * Math.max(Math.abs(browserValue), Math.abs(nativeValue)),
      );
      if (Math.abs(browserValue - nativeValue) > tolerance)
        errors.push(`consumed editor token differs: ${key}`);
    } else if (browserValue !== nativeValue) {
      errors.push(`consumed editor token differs: ${key}`);
    }
  }
  return errors;
}

const isRepoRelativeZuiPath = (path: unknown): path is string =>
  typeof path === 'string' &&
  path.length > 0 &&
  path.endsWith('.zui') &&
  !path.startsWith('/') &&
  !/^[a-zA-Z]:/.test(path) &&
  !path.includes('\\') &&
  !path.split('/').some((segment) => !segment || segment === '.' || segment === '..');

/** Derive the loaded source-file closure from the actual case-projected host. */
export function deriveExpectedRuntimeSourceFiles(
  preparedDocument: unknown,
  additionalRootPaths: string[],
  catalogFingerprints: RuntimeSourceFingerprint[],
): { sources: RuntimeSourceFingerprint[]; errors: string[] } {
  const errors: string[] = [];
  if (!isRecord(preparedDocument))
    return {
      sources: [],
      errors: ['prepared workbench source closure is unavailable'],
    };
  const paths = new Set<string>();
  const addPath = (value: unknown, label: string): void => {
    if (!isRepoRelativeZuiPath(value)) {
      errors.push(`${label} is not a repository-relative .zui source path`);
      return;
    }
    paths.add(value);
  };
  for (const path of additionalRootPaths) addPath(path, 'workbench source root');

  const dependencies = preparedDocument['penpot_dependency_sources'];
  if (!Array.isArray(dependencies)) {
    errors.push('prepared workbench source dependency closure is unavailable');
  } else {
    for (const path of dependencies) {
      if (typeof path === 'string' && path.endsWith('.zui'))
        addPath(path, 'prepared source dependency');
      else if (
        typeof path !== 'string' ||
        path.startsWith('/') ||
        /^[a-zA-Z]:/.test(path) ||
        path.includes('\\') ||
        path.split('/').some((segment) => !segment || segment === '.' || segment === '..')
      )
        errors.push('prepared source dependency is not a contained repo-relative path');
    }
  }
  const themeSource = preparedDocument['penpot_host_theme_source'];
  if (themeSource !== undefined) addPath(themeSource, 'prepared theme source');

  const presentation = preparedDocument['penpot_review_workbench_presentation'];
  if (!isRecord(presentation) || !isRecord(presentation['sourceFingerprint'])) {
    errors.push('workbench presentation source fingerprint is unavailable');
  } else {
    addPath(
      presentation['sourceFingerprint']['sourcePath'],
      'workbench presentation source fingerprint',
    );
  }

  const nodes = preparedDocument['nodes'];
  if (!isRecord(nodes)) {
    errors.push('prepared workbench nodes are unavailable for source closure');
  } else {
    for (const [nodeId, rawNode] of Object.entries(nodes)) {
      if (!isRecord(rawNode)) {
        errors.push(`prepared node ${nodeId} is malformed for source closure`);
        continue;
      }
      const sourcePath = rawNode['penpot_review_source_path'];
      if (sourcePath !== undefined && sourcePath !== null)
        addPath(sourcePath, `prepared node ${nodeId} source owner`);
    }
  }

  const fingerprintByPath = new Map<string, string>();
  for (const { sourcePath, sha256 } of catalogFingerprints) {
    const previous = fingerprintByPath.get(sourcePath);
    if (previous && previous !== sha256)
      errors.push(`conflicting catalog source fingerprints: ${sourcePath}`);
    fingerprintByPath.set(sourcePath, sha256);
  }
  const sources = [...paths]
    .sort()
    .map((sourcePath) => {
      const sha256 = fingerprintByPath.get(sourcePath);
      if (!validHash(sha256))
        errors.push(`prepared source is absent from catalog fingerprints: ${sourcePath}`);
      return { sourcePath, sha256: validHash(sha256) ? sha256 : '' };
    });
  if (!sources.length)
    errors.push('prepared workbench source closure has no source files');
  return { sources, errors };
}

/** Find the actual product template documents mounted by the case snapshot. */
export function expectedWorkbenchRuntimeDocumentPaths(
  presentationValue: unknown,
): { sourcePaths: string[]; errors: string[] } {
  const errors: string[] = [];
  const sourcePaths = new Set([
    'zircon_editor/assets/ui/editor/host/workbench_shell.zui',
    'zircon_editor/assets/ui/editor/windows/workbench_window.zui',
  ]);
  if (!isRecord(presentationValue))
    return {
      sourcePaths: [...sourcePaths].sort(),
      errors: ['workbench presentation is unavailable for runtime document roots'],
    };

  const documents = presentationValue['documents'];
  if (!isRecord(documents) || !Array.isArray(documents['items'])) {
    errors.push('workbench active document inventory is unavailable');
  } else {
    const activeId = documents['activeId'];
    if (activeId !== null) {
      const matches = documents['items'].filter(
        (item) => isRecord(item) && item['id'] === activeId,
      );
      if (matches.length !== 1) {
        errors.push('workbench active document does not resolve exactly once');
      } else {
        const sourcePath = (matches[0] as Record<string, unknown>)['sourcePath'];
        if (sourcePath !== null) {
          if (!isRepoRelativeZuiPath(sourcePath))
            errors.push('workbench active document source is not a repository .zui path');
          else sourcePaths.add(sourcePath);
        }
      }
    }
  }

  const drawers = presentationValue['drawers'];
  if (!Array.isArray(drawers)) {
    errors.push('workbench drawer inventory is unavailable');
  } else {
    for (const [index, drawer] of drawers.entries()) {
      if (!isRecord(drawer)) {
        errors.push(`workbench drawer ${index} is malformed`);
        continue;
      }
      if (drawer['visible'] !== true || drawer['activeTabId'] === null) continue;
      if (!Array.isArray(drawer['tabs'])) {
        errors.push(`workbench drawer ${index} tabs are unavailable`);
        continue;
      }
      const matches = drawer['tabs'].filter(
        (tab) => isRecord(tab) && tab['id'] === drawer['activeTabId'],
      );
      if (matches.length !== 1) {
        errors.push(`workbench drawer ${index} active tab does not resolve exactly once`);
        continue;
      }
      const sourcePath = (matches[0] as Record<string, unknown>)['sourcePath'];
      if (sourcePath !== null) {
        if (!isRepoRelativeZuiPath(sourcePath))
          errors.push(`workbench drawer ${index} source is not a repository .zui path`);
        else sourcePaths.add(sourcePath);
      }
    }
  }
  return { sourcePaths: [...sourcePaths].sort(), errors };
}

export interface BrowserRuntimeSourceFile extends RuntimeLoadedSourceFile {}

/** Seal independently read browser source files against the prepared closure. */
export function buildBrowserRuntimeLoadedSourcesAudit(
  expectedSources: RuntimeSourceFingerprint[],
  loadedFiles: BrowserRuntimeSourceFile[],
  documentSourcePaths: string[],
  initialErrors: string[] = [],
): { audit: RuntimeLoadedSources; errors: string[] } {
  const errors = [...initialErrors];
  const expectedByPath = new Map(
    expectedSources.map(({ sourcePath, sha256 }) => [sourcePath, sha256]),
  );
  const fileByPath = new Map<string, BrowserRuntimeSourceFile>();
  for (const file of loadedFiles) {
    if (fileByPath.has(file.sourcePath)) {
      errors.push(`browser runtime source is duplicated: ${file.sourcePath}`);
      continue;
    }
    fileByPath.set(file.sourcePath, file);
    if (
      !isRepoRelativeZuiPath(file.sourcePath) ||
      !nonemptyString(file.assetId) ||
      !/^res:\/\//.test(file.resourceUri) ||
      !nonemptyString(file.physicalPath) ||
      !(isAbsolute(file.physicalPath) || win32.isAbsolute(file.physicalPath)) ||
      !validHash(file.sha256) ||
      file.catalogMatches !== true ||
      file.currentFileMatches !== true
    )
      errors.push(`browser runtime source is not current and catalog-backed: ${file.sourcePath}`);
  }
  if (
    expectedSources.length === 0 ||
    expectedByPath.size !== expectedSources.length ||
    fileByPath.size !== expectedByPath.size ||
    [...expectedByPath.keys()].some((path) => !fileByPath.has(path)) ||
    [...fileByPath.keys()].some((path) => !expectedByPath.has(path))
  )
    errors.push('browser runtime source closure differs from prepared workbench');
  for (const [sourcePath, expectedHash] of expectedByPath) {
    const file = fileByPath.get(sourcePath);
    if (!file) continue;
    if (!validHash(expectedHash) || file.sha256 !== expectedHash)
      errors.push(`browser runtime source hash differs from catalog: ${sourcePath}`);
    const assetMarker = '/assets/';
    const markerIndex = sourcePath.indexOf(assetMarker);
    const expectedResourceUri = markerIndex < 0
      ? null
      : `res://${sourcePath.slice(markerIndex + assetMarker.length)}`;
    if (file.resourceUri !== expectedResourceUri)
      errors.push(`browser runtime resource URI does not map to its source path: ${sourcePath}`);
  }

  const documentIds: string[] = [];
  for (const sourcePath of documentSourcePaths) {
    const file = fileByPath.get(sourcePath);
    if (!file) {
      errors.push(`browser runtime document has no prepared source file: ${sourcePath}`);
      continue;
    }
    documentIds.push(file.resourceUri);
  }
  if (
    documentIds.length === 0 ||
    new Set(documentIds).size !== documentIds.length ||
    documentIds.some((id) => !nonemptyString(id))
  )
    errors.push('browser runtime document IDs are incomplete or ambiguous');

  const audit: RuntimeLoadedSources = {
    complete: errors.length === 0,
    documentIds,
    files: loadedFiles,
    unresolvedImports: [],
  };
  return { audit, errors };
}

function readRuntimeLoadedSources(
  geometry: unknown,
): unknown {
  if (!isRecord(geometry)) return undefined;
  const sourceIdentityProvenance = geometry['sourceIdentityProvenance'];
  if (!isRecord(sourceIdentityProvenance)) return undefined;
  return sourceIdentityProvenance['runtimeLoadedSources'];
}

function validateRuntimeLoadedSources(
  geometry: unknown,
  renderer: 'penpot' | 'engine',
  expectedSources: RuntimeSourceFingerprint[],
): { receipt: RuntimeLoadedSources | null; errors: string[] } {
  const errors: string[] = [];
  const value = readRuntimeLoadedSources(geometry);
  if (
    !isRecord(value) ||
    !exactKeys(value, ['complete', 'documentIds', 'files', 'unresolvedImports'])
  )
    return {
      receipt: null,
      errors: [`${renderer}: missing or unsupported runtime source closure`],
    };
  if (value['complete'] !== true)
    errors.push(`${renderer}: runtime source closure is incomplete`);
  if (
    !Array.isArray(value['documentIds']) ||
    !value['documentIds'].length ||
    value['documentIds'].some((id) => !nonemptyString(id)) ||
    new Set(value['documentIds']).size !== value['documentIds'].length
  )
    errors.push(`${renderer}: runtime source document IDs are incomplete`);
  if (!Array.isArray(value['files']))
    return {
      receipt: null,
      errors: [...errors, `${renderer}: runtime source files are unavailable`],
    };
  if (!Array.isArray(value['unresolvedImports']) || value['unresolvedImports'].length)
    errors.push(`${renderer}: unresolved runtime source imports`);

  const files = new Map<string, RuntimeLoadedSourceFile>();
  for (const [index, rawFile] of value['files'].entries()) {
    if (!isRecord(rawFile) || !exactKeys(rawFile, RUNTIME_SOURCE_FIELDS)) {
      errors.push(`${renderer}: runtime source file ${index} is malformed`);
      continue;
    }
    const file = rawFile as unknown as RuntimeLoadedSourceFile;
    const sourcePath = typeof file.sourcePath === 'string' ? file.sourcePath : '';
    const assetMarker = '/assets/';
    const markerIndex = sourcePath.indexOf(assetMarker);
    const expectedResourceUri = markerIndex < 0
      ? null
      : `res://${sourcePath.slice(markerIndex + assetMarker.length)}`;
    if (
      !nonemptyString(file.assetId) ||
      !isRepoRelativeZuiPath(file.sourcePath) ||
      !/^res:\/\//.test(file.resourceUri) ||
      file.resourceUri !== expectedResourceUri ||
      !nonemptyString(file.physicalPath) ||
      !(isAbsolute(file.physicalPath) || win32.isAbsolute(file.physicalPath)) ||
      !validHash(file.sha256) ||
      file.catalogMatches !== true ||
      file.currentFileMatches !== true
    ) {
      errors.push(`${renderer}: loaded source is not current and catalog-backed`);
      continue;
    }
    if (files.has(file.sourcePath)) {
      errors.push(`${renderer}: duplicate loaded source ${file.sourcePath}`);
      continue;
    }
    files.set(file.sourcePath, file);
  }

  if (!Array.isArray(value['documentIds'])) {
    // The earlier structural error is enough; do not interpret malformed IDs.
  } else {
    for (const documentId of value['documentIds']) {
      if (
        !value['files'].some(
          (rawFile) =>
            isRecord(rawFile) &&
            (rawFile['resourceUri'] === documentId ||
              rawFile['sourcePath'] === documentId ||
              rawFile['assetId'] === documentId),
        )
      )
        errors.push(`${renderer}: runtime document ID has no source file ${String(documentId)}`);
    }
  }

  const expectedByPath = new Map(expectedSources.map((item) => [item.sourcePath, item.sha256]));
  if (
    !expectedSources.length ||
    expectedByPath.size !== expectedSources.length ||
    files.size !== expectedByPath.size ||
    [...expectedByPath.keys()].some((path) => !files.has(path)) ||
    [...files.keys()].some((path) => !expectedByPath.has(path))
  ) {
    errors.push(`${renderer}: loaded source closure differs from prepared host`);
  }
  for (const [sourcePath, expectedHash] of expectedByPath) {
    const actual = files.get(sourcePath);
    if (!actual) continue;
    if (!validHash(expectedHash) || actual.sha256 !== expectedHash)
      errors.push(`${renderer}: loaded source hash differs from catalog: ${sourcePath}`);
  }
  return {
    receipt: value as unknown as RuntimeLoadedSources,
    errors,
  };
}

/** Validate the full source closure and compare the two independently loaded closures. */
export function runtimeLoadedSourceErrors(
  penpotGeometry: unknown,
  engineGeometry: unknown,
  expectedSources: RuntimeSourceFingerprint[],
): string[] {
  const penpot = validateRuntimeLoadedSources(
    penpotGeometry,
    'penpot',
    expectedSources,
  );
  const engine = validateRuntimeLoadedSources(
    engineGeometry,
    'engine',
    expectedSources,
  );
  const errors = [...penpot.errors, ...engine.errors];
  if (!penpot.receipt || !engine.receipt) return errors;
  const penpotDocumentIds = [...penpot.receipt.documentIds].sort();
  const engineDocumentIds = [...engine.receipt.documentIds].sort();
  if (
    penpotDocumentIds.length !== engineDocumentIds.length ||
    penpotDocumentIds.some((id, index) => id !== engineDocumentIds[index])
  )
    errors.push('loaded source document inventories differ between renderers');
  const filesByPath = (receipt: RuntimeLoadedSources) =>
    new Map(receipt.files.map((file) => [file.sourcePath, file]));
  const penpotFiles = filesByPath(penpot.receipt);
  const engineFiles = filesByPath(engine.receipt);
  if (
    penpotFiles.size !== engineFiles.size ||
    [...penpotFiles].some(([path, penpotFile]) => {
      const engineFile = engineFiles.get(path);
      return (
        !engineFile ||
        engineFile.sha256 !== penpotFile.sha256 ||
        engineFile.assetId !== penpotFile.assetId ||
        engineFile.resourceUri !== penpotFile.resourceUri
      );
    })
  )
    errors.push('loaded source file identities differ between renderers');
  return errors;
}
