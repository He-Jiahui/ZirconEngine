use crate::assets::AssetCatalogEntry;
use crate::engines::SourceEngineInstall;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::projects::{ProjectMetadataMap, RecentProject};
use crate::settings::HubSettings;
use crate::team::TeamOverview;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{
    HubActionRecord, HubPage, HubScope, ProjectFilterMode, ProjectSortMode, ProjectSubpage,
    ProjectViewMode, TaskStatus,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProjectAvailabilitySnapshot {
    by_path: HashMap<PathBuf, bool>,
}

impl ProjectAvailabilitySnapshot {
    pub(crate) fn capture(projects: &[RecentProject]) -> Self {
        Self::capture_with_selected(projects, None)
    }

    pub(crate) fn capture_with_selected(
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> Self {
        Self::capture_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    pub(crate) fn synchronize(&mut self, projects: &[RecentProject]) -> bool {
        self.synchronize_with_selected(projects, None)
    }

    pub(crate) fn synchronize_with_selected(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> bool {
        self.synchronize_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    pub(crate) fn refresh(&mut self, projects: &[RecentProject]) -> bool {
        self.refresh_with_selected(projects, None)
    }

    pub(crate) fn refresh_with_selected(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
    ) -> bool {
        self.refresh_with_selected_and_probe(projects, selected_path, Path::exists)
    }

    pub(crate) fn path_exists(&self, path: &Path) -> bool {
        self.by_path.get(path).copied().unwrap_or(false)
    }

    fn capture_with_probe(
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> Self {
        Self::capture_with_selected_and_probe(projects, None, &mut probe)
    }

    fn capture_with_selected_and_probe(
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> Self {
        let mut snapshot = Self::default();
        snapshot.synchronize_with_selected_and_probe(projects, selected_path, &mut probe);
        snapshot
    }

    fn synchronize_with_probe(
        &mut self,
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        self.synchronize_with_selected_and_probe(projects, None, &mut probe)
    }

    fn synchronize_with_selected_and_probe(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        let selected_outside_recents =
            selected_path.is_some_and(|path| !projects.iter().any(|project| project.path == path));
        let expected_len = projects.len() + usize::from(selected_outside_recents);
        if self.by_path.len() == expected_len
            && projects
                .iter()
                .all(|project| self.by_path.contains_key(&project.path))
            && selected_path.is_none_or(|path| self.by_path.contains_key(path))
        {
            return false;
        }

        let mut synchronized = HashMap::with_capacity(expected_len);
        for project in projects {
            let exists = self
                .by_path
                .remove(&project.path)
                .unwrap_or_else(|| probe(&project.path));
            synchronized.insert(project.path.clone(), exists);
        }
        if let Some(path) = selected_path {
            if !synchronized.contains_key(path) {
                let exists = self.by_path.remove(path).unwrap_or_else(|| probe(path));
                synchronized.insert(path.to_path_buf(), exists);
            }
        }
        self.by_path = synchronized;
        true
    }

    fn refresh_with_probe(
        &mut self,
        projects: &[RecentProject],
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        self.refresh_with_selected_and_probe(projects, None, &mut probe)
    }

    fn refresh_with_selected_and_probe(
        &mut self,
        projects: &[RecentProject],
        selected_path: Option<&Path>,
        mut probe: impl FnMut(&Path) -> bool,
    ) -> bool {
        let refreshed = Self::capture_with_selected_and_probe(projects, selected_path, &mut probe);
        if *self == refreshed {
            return false;
        }
        *self = refreshed;
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HubSnapshot {
    pub selected_page: HubPage,
    pub project_filter: ProjectFilterMode,
    pub project_sort: ProjectSortMode,
    pub project_view_mode: ProjectViewMode,
    pub project_subpage: ProjectSubpage,
    pub search_query: String,
    pub selected_project_path: Option<PathBuf>,
    pub new_project_name: String,
    pub selected_template_id: String,
    pub new_project_location: PathBuf,
    pub new_project_engine_id: Option<String>,
    pub pending_delete_project_path: Option<PathBuf>,
    pub task_status: TaskStatus,
    pub queued_background_actions: usize,
    pub recent_projects: Vec<RecentProject>,
    pub project_metadata: ProjectMetadataMap,
    pub assets: Vec<AssetCatalogEntry>,
    pub learn_resources: Vec<LearnCatalogEntry>,
    pub plugins: Vec<PluginCatalogEntry>,
    pub team: TeamOverview,
    pub action_history: Vec<HubActionRecord>,
    pub engines: Vec<SourceEngineInstall>,
    pub active_engine_id: Option<String>,
    pub settings: HubSettings,
    pub settings_draft: HubSettings,
}

impl HubSnapshot {
    pub fn scope(&self) -> HubScope {
        HubScope::resolve(
            self.selected_project_path.as_deref(),
            &self.recent_projects,
            &self.project_metadata,
            &self.engines,
            self.active_engine_id.as_deref(),
        )
    }

    pub fn filtered_recent_projects(&self) -> Vec<RecentProject> {
        let availability = ProjectAvailabilitySnapshot::capture(&self.recent_projects);
        self.filtered_recent_projects_with_availability(&availability)
    }

    pub(crate) fn filtered_recent_projects_with_availability(
        &self,
        availability: &ProjectAvailabilitySnapshot,
    ) -> Vec<RecentProject> {
        let query = self.search_query.trim().to_ascii_lowercase();
        let mut projects: Vec<_> = self
            .recent_projects
            .iter()
            .filter(|project| self.project_filter.includes(project, availability))
            .filter(|project| query.is_empty() || project_matches_query(project, &query))
            .cloned()
            .collect();

        match self.project_sort {
            ProjectSortMode::LastModified => projects
                .sort_by(|left, right| right.last_opened_unix_ms.cmp(&left.last_opened_unix_ms)),
            ProjectSortMode::Name => {
                projects.sort_by_key(|project| project_display_name(project).to_ascii_lowercase());
            }
        }

        projects
    }
}

impl ProjectFilterMode {
    fn includes(self, project: &RecentProject, availability: &ProjectAvailabilitySnapshot) -> bool {
        match self {
            Self::All => true,
            Self::Existing => availability.path_exists(&project.path),
            Self::Missing => !availability.path_exists(&project.path),
        }
    }
}

fn project_matches_query(project: &RecentProject, query: &str) -> bool {
    project.summary.name.to_ascii_lowercase().contains(query)
        || project
            .path
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains(query)
}

fn project_display_name(project: &RecentProject) -> String {
    if project.summary.name.trim().is_empty() {
        return project
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
    }
    project.summary.name.clone()
}

#[cfg(test)]
mod tests {
    use crate::settings::HubSettings;
    use std::{fs, hint::black_box, time::Instant};

    use crate::state::{
        HubPage, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
    };

    use super::*;

    #[test]
    fn filtered_recent_projects_sorts_by_selected_mode() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::Name,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects: vec![
                RecentProject::fixture("Zeta", "E:/Projects/Zeta", 30),
                RecentProject::fixture("Alpha", "E:/Projects/Alpha", 10),
            ],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
        };

        let projects = snapshot.filtered_recent_projects();

        assert_eq!(projects[0].summary.name, "Alpha");
        assert_eq!(projects[1].summary.name, "Zeta");
    }

    #[test]
    fn filtered_recent_projects_applies_path_filter_before_sorting() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-filter-test-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let existing = root.join("Existing");
        let missing = root.join("Missing");
        fs::create_dir_all(&existing).unwrap();
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::Existing,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: root.join("Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects: vec![
                RecentProject::fixture("Missing", missing.clone(), 30),
                RecentProject::fixture("Existing", existing.clone(), 10),
            ],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
        };

        let projects = snapshot.filtered_recent_projects();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].summary.name, "Existing");
    }

    #[test]
    fn fixture_named_projects_follow_real_path_existence() {
        let missing_fixture_path = format!(
            "C:/Zircon{}/ElysiumMissing-{}",
            "Projects",
            std::process::id()
        );
        let project = RecentProject::fixture(
            format!("{} {}", "Elysium", "Chronicles"),
            &missing_fixture_path,
            10,
        );
        let availability = ProjectAvailabilitySnapshot::capture(std::slice::from_ref(&project));

        assert!(!ProjectFilterMode::Existing.includes(&project, &availability));
        assert!(ProjectFilterMode::Missing.includes(&project, &availability));
    }

    #[test]
    fn hub04_project_availability_synchronize_only_probes_added_paths() {
        let first = RecentProject::fixture("First", "E:/Projects/First", 20);
        let second = RecentProject::fixture("Second", "E:/Projects/Second", 10);
        let mut projects = vec![first, second];
        let mut probes = 0usize;
        let mut availability = ProjectAvailabilitySnapshot::default();

        assert!(availability.synchronize_with_probe(&projects, |_| {
            probes += 1;
            false
        }));
        assert_eq!(probes, 2);
        assert!(!availability.synchronize_with_probe(&projects, |_| {
            probes += 1;
            false
        }));
        assert_eq!(probes, 2);

        projects.push(RecentProject::fixture("Third", "E:/Projects/Third", 5));
        assert!(availability.synchronize_with_probe(&projects, |_| {
            probes += 1;
            true
        }));
        assert_eq!(probes, 3);
        assert!(availability.path_exists(&projects[2].path));
    }

    #[test]
    fn hub04_project_availability_caches_selected_path_outside_recents() {
        let project = RecentProject::fixture("Recent", "E:/Projects/Recent", 20);
        let selected_path = PathBuf::from("E:/Projects/SelectedOutsideRecents");
        let mut probes = 0usize;
        let mut availability = ProjectAvailabilitySnapshot::default();

        assert!(availability.synchronize_with_selected_and_probe(
            std::slice::from_ref(&project),
            Some(&selected_path),
            |path| {
                probes += 1;
                path == selected_path
            },
        ));
        assert_eq!(probes, 2);
        assert!(availability.path_exists(&selected_path));

        assert!(!availability.synchronize_with_selected_and_probe(
            std::slice::from_ref(&project),
            Some(&selected_path),
            |_| panic!("unchanged paths must not be probed again"),
        ));
    }

    #[test]
    fn hub04_project_availability_filter_uses_cached_snapshot() {
        let existing = RecentProject::fixture("Existing", "E:/Projects/Existing", 10);
        let missing = RecentProject::fixture("Missing", "E:/Projects/Missing", 20);
        let projects = vec![missing, existing.clone()];
        let snapshot = snapshot_with_recent_projects(ProjectFilterMode::Existing, projects.clone());
        let availability = ProjectAvailabilitySnapshot::capture_with_probe(&projects, |path| {
            path == existing.path
        });

        let filtered = snapshot.filtered_recent_projects_with_availability(&availability);

        assert_eq!(filtered, vec![existing]);
    }

    #[test]
    #[ignore = "managed release performance contract"]
    fn hub04_project_availability_filter_release_benchmark_evidence() {
        const PROJECT_COUNT: usize = 10_000;
        const SAMPLE_PAIRS: usize = 21;
        const THRESHOLD_PERCENT: u64 = 40;

        let root = std::env::temp_dir().join(format!(
            "zircon-hub-availability-benchmark-{}",
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let projects = (0..PROJECT_COUNT)
            .map(|index| {
                RecentProject::fixture(
                    format!("Missing {index}"),
                    root.join(format!("missing-{index}")),
                    index as u64,
                )
            })
            .collect::<Vec<_>>();
        let snapshot = snapshot_with_recent_projects(ProjectFilterMode::Existing, projects.clone());
        let mut availability = ProjectAvailabilitySnapshot::capture(&projects);

        assert!(legacy_filtered_recent_projects(&snapshot).is_empty());
        assert!(snapshot
            .filtered_recent_projects_with_availability(&availability)
            .is_empty());

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                let started = Instant::now();
                black_box(legacy_filtered_recent_projects(black_box(&snapshot)));
                elapsed_nanos(started)
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                black_box(availability.synchronize(&projects));
                black_box(
                    black_box(&snapshot)
                        .filtered_recent_projects_with_availability(black_box(&availability)),
                );
                elapsed_nanos(started)
            };
            let (legacy_ns, optimized_ns) = if pair % 2 == 0 {
                (measure_legacy(), measure_optimized())
            } else {
                let optimized_ns = measure_optimized();
                (measure_legacy(), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT hub04_project_availability_cache sample_pairs=21 \
             projects=10000 legacy_filesystem_probes_per_projection=10000 \
             optimized_filesystem_probes_per_projection=0 threshold_percent=40 \
             legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} \
             optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} \
             improvement_percent={improvement_percent} legacy_ns={} optimized_ns={}",
            join_samples(&legacy_samples),
            join_samples(&optimized_samples),
        );
        let _ = fs::remove_dir_all(&root);

        assert!(
            improvement_percent >= THRESHOLD_PERCENT,
            "optimized P95 improvement {improvement_percent}% misses {THRESHOLD_PERCENT}% gate"
        );
    }

    #[test]
    fn snapshot_scope_exposes_selected_project_without_latest_recent_fallback() {
        let snapshot = HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: Some(PathBuf::from("E:/Projects/Missing")),
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects: vec![RecentProject::fixture("Latest", "E:/Projects/Latest", 20)],
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
        };

        let scope = snapshot.scope();

        assert!(scope.has_stale_selected_project());
        assert!(scope.selected_or_latest_project().is_none());
    }

    fn snapshot_with_recent_projects(
        project_filter: ProjectFilterMode,
        recent_projects: Vec<RecentProject>,
    ) -> HubSnapshot {
        HubSnapshot {
            selected_page: HubPage::Projects,
            project_filter,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            new_project_name: String::new(),
            selected_template_id: "renderable-empty".to_string(),
            new_project_location: PathBuf::from("E:/Projects"),
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            queued_background_actions: 0,
            recent_projects,
            project_metadata: ProjectMetadataMap::new(),
            assets: Vec::new(),
            learn_resources: Vec::new(),
            plugins: Vec::new(),
            team: TeamOverview::empty(),
            action_history: Vec::new(),
            engines: Vec::new(),
            active_engine_id: None,
            settings: HubSettings::default(),
            settings_draft: HubSettings::default(),
        }
    }

    fn legacy_filtered_recent_projects(snapshot: &HubSnapshot) -> Vec<RecentProject> {
        let query = snapshot.search_query.trim().to_ascii_lowercase();
        let mut projects = snapshot
            .recent_projects
            .iter()
            .filter(|project| match snapshot.project_filter {
                ProjectFilterMode::All => true,
                ProjectFilterMode::Existing => project.path.exists(),
                ProjectFilterMode::Missing => !project.path.exists(),
            })
            .filter(|project| query.is_empty() || project_matches_query(project, &query))
            .cloned()
            .collect::<Vec<_>>();
        match snapshot.project_sort {
            ProjectSortMode::LastModified => projects
                .sort_by(|left, right| right.last_opened_unix_ms.cmp(&left.last_opened_unix_ms)),
            ProjectSortMode::Name => {
                projects.sort_by_key(|project| project_display_name(project).to_ascii_lowercase());
            }
        }
        projects
    }

    fn elapsed_nanos(started: Instant) -> u64 {
        started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u64]) -> String {
        samples
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
