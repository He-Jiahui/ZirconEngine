export type StatusTone = "running" | "success" | "warning" | "error" | "neutral";

export interface HubStatusPill {
  id: string;
  label: string;
  tone: StatusTone;
}

export interface HubTaskSummary {
  label: string;
  detail: string;
  tone: StatusTone;
  running: boolean;
  recovery: string | null;
  operation: string;
  progressPercent: number;
  taskId: number;
  queued: number;
}

export interface HubProjectSummary {
  id: string;
  name: string;
  path: string;
  modified: string;
  engineVersion: string;
  platform: string;
  coverId: string;
}

export interface HubRecentProject {
  id: string;
  name: string;
  engineVersion: string;
  modified: string;
  location: string;
  coverId: string;
  pinned: boolean;
}

export interface HubProjectDetail {
  id: string;
  name: string;
  path: string;
  modified: string;
  engineVersion: string;
  platform: string;
  coverId: string;
  pinned: boolean;
  engineId: string | null;
  templateId: string | null;
  templateLabel: string;
  exists: boolean;
  status: string;
  pendingDelete: boolean;
}

export interface HubProjectTemplate {
  id: string;
  title: string;
  optionLabel: string;
  category: string;
  description: string;
  enabled: boolean;
  status: string;
  disabledReason: string | null;
}

export interface HubQuickAction {
  id: HubActionId;
  title: string;
  detail: string;
  icon: string;
  enabled: boolean;
}

export interface HubSourceBuildHistoryItem {
  id: string;
  status: string;
  statusTone: StatusTone;
  profile: string;
  jobs: number | null;
  detail: string;
  secondaryDetail: string;
  logExcerpt: string;
  commandLine: string[];
  outputDir: string;
  finished: string;
}

export interface HubSourceEngineSummary {
  id: string;
  name: string;
  sourcePath: string;
  outputPath: string;
  status: string;
  active: boolean;
  buildHistory: HubSourceBuildHistoryItem[];
}

export interface HubAssetItem {
  id: string;
  name: string;
  kind: string;
  detail: string;
  source: string;
  sourceKey: string;
  size: string;
  path: string;
}

export interface HubPluginItem {
  id: string;
  displayName: string;
  description: string;
  category: string;
  maturity: string;
  maturityTone: StatusTone;
  scope: string;
  scopeKey: string;
  editorScoped: boolean;
  moduleCount: number;
  defaultPackaging: string[];
  packageRoot: string;
  manifestPath: string;
}

export interface HubLearnItem {
  id: string;
  title: string;
  category: string;
  categoryKey: string;
  source: string;
  sourceKey: string;
  summary: string;
  path: string;
}

export interface HubTeamMember {
  id: string;
  name: string;
  email: string;
  commits: number;
  commitsLabel: string;
}

export interface HubTeamSummary {
  repositoryPath: string;
  identityName: string;
  identityEmail: string;
  repositoryAvailable: boolean;
  members: HubTeamMember[];
}

export type HubActionHistoryKind =
  | "create-project"
  | "import-project"
  | "remove-project"
  | "delete-project"
  | "build-editor-runtime"
  | "open-editor"
  | "package-project"
  | "install-project"
  | "open-resource"
  | "open-output";

export interface HubActionHistoryDetailRow {
  id: string;
  title: string;
  detail: string;
}

export interface HubActionHistoryItem {
  id: string;
  kind: HubActionHistoryKind;
  action: string;
  status: string;
  tone: StatusTone;
  target: string;
  detail: string;
  logExcerpt: string;
  finished: string;
  recovery: string | null;
  processId: number | null;
  commandLine: string[];
  outputDir: string | null;
  detailRows: HubActionHistoryDetailRow[];
}

export interface HubSettingsHealthRow {
  id: string;
  title: string;
  detail: string;
  meta: string;
  state: string;
  selected: boolean;
}

export interface HubSettingsHealthSummary {
  label: string;
  detail: string;
  tone: StatusTone;
  completion: number;
  rows: HubSettingsHealthRow[];
}

export interface HubSettingsTabText {
  value: string;
  label: string;
}

export interface HubSettingsOptionText {
  value: string;
  label: string;
}

export interface HubSettingsFieldLabels {
  pythonPath: string;
  cargoPath: string;
  rustupPath: string;
  defaultProjectDir: string;
  defaultSourceDir: string;
  defaultBuildOutputDir: string;
  defaultDeviceInstallDir: string;
  buildProfile: string;
  jobs: string;
  language: string;
  releaseBuild: string;
  localizedUi: string;
}

export interface HubSettingsText {
  heading: string;
  projectsButton: string;
  saveButton: string;
  discardButton: string;
  restoreDefaultsButton: string;
  buildDefaultsPanel: string;
  configurationPathsPanel: string;
  sourceEnginesPanel: string;
  pathDefaultsPanel: string;
  advancedConfigurationPanel: string;
  configurationHealthPanel: string;
  activeSourceEnginePanel: string;
  completenessLabel: string;
  jobCountSingularTemplate: string;
  jobCountPluralTemplate: string;
  tabs: HubSettingsTabText[];
  buildProfileOptions: HubSettingsOptionText[];
  languageOptions: HubSettingsOptionText[];
  labels: HubSettingsFieldLabels;
}

export interface HubSettingsSummary {
  pythonPath: string;
  cargoPath: string;
  rustupPath: string;
  defaultProjectDir: string;
  defaultSourceDir: string;
  defaultBuildOutputDir: string;
  defaultDeviceInstallDir: string;
  buildProfile: string;
  buildProfileLabel: string;
  languageLabel: string;
  jobsLabel: string;
  buildProfileDetail: string;
  buildWorkflowDetail: string;
  jobs: number;
  language: string;
  health: HubSettingsHealthSummary;
  text: HubSettingsText;
}

export interface HubNavItemText {
  id: HubPageId;
  label: string;
}

export interface HubShellText {
  productCategory: string;
  workspaceProfile: string;
  activeEngine: string;
  readyFallback: string;
  localDefaults: string;
  noSourceEngineRegistered: string;
  noFallbackEngineConfigured: string;
  manageEngines: string;
  source: string;
  buildOutput: string;
  active: string;
  userAccount: string;
  userAccountDetail: string;
  preferences: string;
  preferencesDetail: string;
  documentation: string;
  documentationDetail: string;
  signOut: string;
  demoModeBadge: string;
  liveUpdatesUnavailable: string;
  liveUpdatesUnavailableDetail: string;
  actionFailed: string;
  actionFailedDetail: string;
  stateRefreshAfterCommand: string;
  checkActionTarget: string;
  navItems: HubNavItemText[];
  engineStatus: string;
  upToDate: string;
  checkForUpdates: string;
  checkForUpdatesDetail: string;
  collapse: string;
  expand: string;
  notifications: string;
  help: string;
  settings: string;
  minimize: string;
  maximize: string;
  close: string;
}

export interface HubActionText {
  importProject: string;
  newProject: string;
  createProject: string;
  close: string;
  dashboard: string;
  browser: string;
  openEditor: string;
  packageProject: string;
  installToDevice: string;
  viewAllProjects: string;
  pinProject: string;
  unpinProject: string;
  removeFromHub: string;
  requestDelete: string;
  cancelDelete: string;
  confirmDelete: string;
  browseFolder: string;
  openResource: string;
}

export interface HubProjectsText {
  title: string;
  browserTitle: string;
  detailTitle: string;
  searchPlaceholder: string;
  filterAll: string;
  filterExisting: string;
  filterMissing: string;
  sortLastModified: string;
  sortName: string;
  gridView: string;
  listView: string;
  noProjectsFound: string;
  searchFiltersEmpty: string;
  noRecentProjectMatches: string;
  projectBrowser: string;
  recentProjects: string;
  quickActions: string;
  sourceEngines: string;
  allProjects: string;
  newProjectDialog: string;
  projectName: string;
  location: string;
  noProjectSelected: string;
  chooseProjectFromBrowser: string;
  status: string;
  ready: string;
  pathUnavailable: string;
  engine: string;
  projectBinding: string;
  lastModified: string;
  projectPin: string;
  pinned: string;
  unpinned: string;
  noTemplate: string;
  overview: string;
  files: string;
  actions: string;
  projectOverview: string;
  projectTree: string;
  projectActions: string;
  sourceEngine: string;
  template: string;
  notRecorded: string;
  platform: string;
  projectId: string;
  content: string;
  available: string;
  missing: string;
  buildOutput: string;
  deviceInstall: string;
  package: string;
  projectManagement: string;
  deleteRequested: string;
  deleteRequestedDetail: string;
  tableName: string;
  tableEngineVersion: string;
  tableLastModified: string;
  tableLocation: string;
  openProjectDetailsLabel: string;
}

export interface HubCommonText {
  overview: string;
  plugins: string;
  activity: string;
  workflow: string;
  history: string;
  outputs: string;
  toolchain: string;
  packages: string;
  installs: string;
  services: string;
  selectedProject: string;
  sourceEngines: string;
  quickActions: string;
  project: string;
  engine: string;
  template: string;
  path: string;
  category: string;
  scope: string;
  target: string;
  finished: string;
  output: string;
  recovery: string;
  log: string;
  command: string;
  operation: string;
  detail: string;
  status: string;
  none: string;
  noProjectSelected: string;
  noSelectedProject: string;
  notConfigured: string;
  configured: string;
  connected: string;
  available: string;
  ready: string;
  local: string;
  reserved: string;
  entries: string;
  actions: string;
  members: string;
  jobs: string;
  entryCountTemplate: string;
  availableCountTemplate: string;
  reservedCountTemplate: string;
  memberCountTemplate: string;
  actionCountTemplate: string;
  noOutputDirectory: string;
  noRecoveryNeeded: string;
  noLogExcerpt: string;
  noCommandRecorded: string;
}

export interface HubEditorText {
  workspaceTree: string;
  editorWorkspace: string;
  selectedProject: string;
  sourceEngines: string;
  sourceBuildHistory: string;
  editorPlugins: string;
  launchTarget: string;
  launchReadiness: string;
  editorPluginScope: string;
  editorActivity: string;
  pluginComingSoonPanel: string;
  noProjectSelectedTitle: string;
  noProjectSelectedDetail: string;
  noEditorPluginsTitle: string;
  noEditorPluginsDetail: string;
  noEditorActivityTitle: string;
  noEditorActivityDetail: string;
  projectAvailable: string;
  sourceEngineRegistered: string;
  editorPluginScopeStatus: string;
  editorPackagingScope: string;
  chooseProject: string;
  noTemplateRecorded: string;
}

export interface HubBuildsText {
  buildProject: string;
  packageProject: string;
  installToDevice: string;
  buildButton: string;
  packageButton: string;
  installButton: string;
  buildWorkflow: string;
  buildHistory: string;
  latestWorkflow: string;
  outputTree: string;
  outputFolders: string;
  buildProfile: string;
  outputRoot: string;
  recentWorkflows: string;
  profile: string;
  jobs: string;
  deviceInstall: string;
  compileDetail: string;
  packageDetail: string;
  installDetail: string;
  noBuildHistory: string;
  noBuildHistoryDetail: string;
  noProjectSelectedDetail: string;
  noWorkflowSelected: string;
  noWorkflowSelectedDetail: string;
  openOutput: string;
}

export interface HubCatalogText {
  searchPlaceholderPrefix: string;
  searchPlaceholderSeparator: string;
  searchPlaceholderSuffix: string;
  entries: string;
  categories: string;
  scopes: string;
  catalogSuffix: string;
  assetsCatalogPanelTitle: string;
  pluginsCatalogPanelTitle: string;
  learnCatalogPanelTitle: string;
  selectedEntry: string;
  catalogTree: string;
  all: string;
  project: string;
  engine: string;
  guides: string;
  reference: string;
  noCatalog: string;
  noScope: string;
  noEntriesFound: string;
  noEntriesFoundDetail: string;
  noCatalogEntrySelected: string;
  noCatalogEntrySelectedDetail: string;
  moduleCountSuffix: string;
  moduleCountTemplate: string;
  comingSoonPanel: string;
}

export interface HubCloudText {
  localDeliveryTree: string;
  localDeliveryTreeDetail: string;
  packageOutput: string;
  packageOutputs: string;
  packageTarget: string;
  packageRoot: string;
  deviceInstall: string;
  deviceInstalls: string;
  installReadiness: string;
  serviceSlots: string;
  reservedServices: string;
  currentStatus: string;
  localPackageHandoff: string;
  reservedLocalServices: string;
  noPackagesRecorded: string;
  noPackagesRecordedDetail: string;
  noInstallsRecorded: string;
  noInstallsRecordedDetail: string;
  deviceInstallFolder: string;
  packageHistory: string;
  packageActionsSuffix: string;
  packageActionCountTemplate: string;
}

export interface HubTeamText {
  repository: string;
  identity: string;
  contributors: string;
  teamMembers: string;
  repositoryIdentity: string;
  teamTree: string;
  actionHistory: string;
  latestAction: string;
  gitName: string;
  gitEmail: string;
  name: string;
  email: string;
  unknownContributor: string;
  noEmailConfigured: string;
  commitSingularSuffix: string;
  commitPluralSuffix: string;
  recentActions: string;
  recentActionCountTemplate: string;
  noTeamMembersFound: string;
  noTeamMembersFoundDetail: string;
  noRecentActions: string;
  noRecentActionsDetail: string;
  noActionSelected: string;
  noActionSelectedDetail: string;
  comingSoonPanel: string;
}

export interface HubUiText {
  shell: HubShellText;
  actions: HubActionText;
  projects: HubProjectsText;
  common: HubCommonText;
  editor: HubEditorText;
  builds: HubBuildsText;
  catalog: HubCatalogText;
  cloud: HubCloudText;
  team: HubTeamText;
}

export interface HubComingSoonEntry {
  id: string;
  category: string;
  categoryLabel: string;
  title: string;
  detail: string;
  status: string;
  meta: string;
  disabled: boolean;
}

export const HUB_ACTION = {
  showPage: "show-page",
  showProjectSubpage: "show-project-subpage",
  searchProjects: "search-projects",
  setProjectFilter: "set-project-filter",
  setProjectSort: "set-project-sort",
  setProjectViewMode: "set-project-view-mode",
  selectProject: "select-project",
  openProjectDetail: "open-project-detail",
  viewAllProjects: "view-all-projects",
  newProject: "new-project",
  updateNewProjectDraft: "update-new-project-draft",
  selectEngine: "select-engine",
  updateSettingsDraft: "update-settings-draft",
  saveSettings: "save-settings",
  discardSettingsDraft: "discard-settings-draft",
  restoreDefaultSettings: "restore-default-settings",
  browseSettingsFolder: "browse-settings-folder",
  createProject: "create-project",
  importProject: "import-project",
  pinProject: "pin-project",
  unpinProject: "unpin-project",
  removeFromHub: "remove-from-hub",
  requestDelete: "request-delete",
  cancelDelete: "cancel-delete",
  confirmDelete: "confirm-delete",
  openResource: "open-resource",
  buildProject: "build-project",
  packageProject: "package-project",
  installDevice: "install-device",
  openEditor: "open-editor",
  openOutputFolder: "open-output-folder",
} as const;

export type HubActionId = (typeof HUB_ACTION)[keyof typeof HUB_ACTION];
export type HubPageId = "projects" | "editor" | "assets" | "builds" | "plugins" | "cloud" | "team" | "learn" | "settings";
export type HubSettingsFolderField =
  | "defaultProjectDir"
  | "defaultSourceDir"
  | "defaultBuildOutputDir"
  | "defaultDeviceInstallDir";

export interface CreateProjectPayload {
  name: string;
  location: string;
  template: string;
  engineId: string | null;
}

export interface NewProjectDraftPayload {
  name: string;
  location: string;
  template: string;
  engineId: string | null;
}

export interface SearchProjectsPayload {
  query: string;
}

export interface ImportProjectPayload {
  path?: string;
  folder?: string;
  engineId?: string;
}

export interface ProjectTargetPayload {
  projectId?: string;
  projectPath?: string;
}

export interface SaveSettingsPayload {
  settings: Partial<HubSettingsSummary>;
}

export interface UpdateSettingsDraftPayload {
  settings: Partial<HubSettingsSummary>;
}

export interface BrowseSettingsFolderPayload {
  field?: HubSettingsFolderField;
  initialDir?: string;
  settings?: Partial<HubSettingsSummary>;
}

export interface OpenResourcePayload {
  resourceId?: string;
  path?: string;
}

export interface OpenOutputFolderPayload {
  outputDir?: string;
  historyId?: string;
}

export interface HubActionPayloadById {
  [HUB_ACTION.searchProjects]: SearchProjectsPayload;
  [HUB_ACTION.updateNewProjectDraft]: NewProjectDraftPayload;
  [HUB_ACTION.createProject]: CreateProjectPayload;
  [HUB_ACTION.importProject]: ImportProjectPayload;
  [HUB_ACTION.pinProject]: ProjectTargetPayload;
  [HUB_ACTION.unpinProject]: ProjectTargetPayload;
  [HUB_ACTION.removeFromHub]: ProjectTargetPayload;
  [HUB_ACTION.requestDelete]: ProjectTargetPayload;
  [HUB_ACTION.cancelDelete]: ProjectTargetPayload;
  [HUB_ACTION.confirmDelete]: ProjectTargetPayload;
  [HUB_ACTION.buildProject]: ProjectTargetPayload;
  [HUB_ACTION.packageProject]: ProjectTargetPayload;
  [HUB_ACTION.installDevice]: ProjectTargetPayload;
  [HUB_ACTION.openEditor]: ProjectTargetPayload;
  [HUB_ACTION.updateSettingsDraft]: UpdateSettingsDraftPayload;
  [HUB_ACTION.saveSettings]: SaveSettingsPayload;
  [HUB_ACTION.browseSettingsFolder]: BrowseSettingsFolderPayload;
  [HUB_ACTION.openResource]: OpenResourcePayload;
  [HUB_ACTION.openOutputFolder]: OpenOutputFolderPayload;
}

export type HubActionPayload<TActionId extends HubActionId = HubActionId> = TActionId extends keyof HubActionPayloadById
  ? HubActionPayloadById[TActionId]
  : undefined;

export type HubActionHandler = <TActionId extends HubActionId>(
  actionId: TActionId,
  targetId?: string,
  payload?: HubActionPayload<TActionId>,
) => void | Promise<void>;

export interface HubShellState {
  productName: string;
  engineVersion: string;
  activePage: string;
  demoMode?: boolean;
  pageTitle: string;
  pageSubtitle: string;
  projectFilter: string;
  projectSort: string;
  projectViewMode: string;
  projectSubpage: string;
  projectTemplates: HubProjectTemplate[];
  searchQuery: string;
  selectedProjectId: string | null;
  activeSourceEngineId: string | null;
  taskSummary: HubTaskSummary;
  taskStatus: HubStatusPill[];
  projects: HubProjectSummary[];
  browserProjects: HubRecentProject[];
  recentProjects: HubRecentProject[];
  selectedProject: HubProjectDetail | null;
  quickActions: HubQuickAction[];
  sourceEngines: HubSourceEngineSummary[];
  assets: HubAssetItem[];
  plugins: HubPluginItem[];
  learnResources: HubLearnItem[];
  team: HubTeamSummary;
  actionHistory: HubActionHistoryItem[];
  comingSoon: HubComingSoonEntry[];
  settings: HubSettingsSummary;
  settingsDraft: HubSettingsSummary | null;
  ui: HubUiText;
}
