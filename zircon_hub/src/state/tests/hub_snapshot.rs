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
    let availability =
        ProjectAvailabilitySnapshot::capture_with_probe(&projects, |path| path == existing.path);

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
        ProjectSortMode::LastModified => {
            projects.sort_by(|left, right| right.last_opened_unix_ms.cmp(&left.last_opened_unix_ms))
        }
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
