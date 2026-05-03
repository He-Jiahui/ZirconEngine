use std::collections::HashMap;
use std::sync::Mutex;

use crate::component_json::parse_component;
use crate::runtime_obstacles::{
    collect_runtime_obstacles, distance_xz, node_intersects_obstacle, recast_carving_obstacles,
    RuntimeObstacle,
};
use crate::settings_hash::navigation_settings_hash;
use crate::settings_validation::validate_navigation_settings;
use zircon_plugin_navigation_recast::{RecastBackend, RecastBakeInput, RecastBakeMeshInput};
use zircon_runtime::asset::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshLinkAsset, NavigationSettingsAsset,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavMeshBakeDiagnostic,
    NavMeshBakeDiagnosticSeverity, NavMeshBakeReport, NavMeshBakeRequest, NavMeshHandle,
    NavMeshModifierDescriptor, NavMeshModifierMode, NavMeshOffMeshLinkDescriptor,
    NavMeshSurfaceDescriptor, NavMeshUseGeometry, NavPathQuery, NavPathResult, NavPathStatus,
    NavRaycastQuery, NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationError,
    NavigationErrorKind, NavigationManager, NavigationRuntimeStats, DEFAULT_AGENT_TYPE,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_MODIFIER_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Mat4, Quat, Real, Transform, Vec3};
use zircon_runtime::scene::components::{ColliderShape, NodeKind, SceneNode};
use zircon_runtime::scene::World;

#[derive(Debug)]
pub struct DefaultNavigationManager {
    backend: RecastBackend,
    state: Mutex<NavigationRuntimeState>,
}

impl DefaultNavigationManager {
    pub fn new() -> Self {
        Self {
            backend: RecastBackend,
            state: Mutex::new(NavigationRuntimeState::default()),
        }
    }

    pub fn active_settings(&self) -> NavigationSettingsAsset {
        self.state
            .lock()
            .expect("navigation state lock poisoned")
            .settings
            .clone()
    }

    fn selected_asset(
        &self,
        query_handle: Option<NavMeshHandle>,
    ) -> Result<NavMeshAsset, NavigationError> {
        let state = self.state.lock().expect("navigation state lock poisoned");
        let handle = query_handle
            .or_else(|| state.loaded.keys().copied().min_by_key(|handle| handle.0))
            .ok_or_else(|| NavigationError::missing_nav_mesh("no nav mesh is loaded"))?;
        state.loaded.get(&handle).cloned().ok_or_else(|| {
            NavigationError::missing_nav_mesh(format!("nav mesh {:?} is not loaded", handle))
        })
    }
}

impl Default for DefaultNavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationManager for DefaultNavigationManager {
    fn bake_surface(
        &self,
        world: &World,
        request: NavMeshBakeRequest,
    ) -> Result<NavMeshBakeReport, NavigationError> {
        let surfaces = collect_surfaces(world);
        let selected_surface = request
            .surface_entity
            .and_then(|entity| surfaces.iter().find(|(id, _)| *id == entity).cloned())
            .or_else(|| surfaces.first().cloned());
        let surface = selected_surface
            .as_ref()
            .map(|(_, surface)| surface.clone())
            .unwrap_or_default();
        let surface_entity = selected_surface.as_ref().map(|(entity, _)| *entity);
        let agent_type = request
            .agent_type
            .clone()
            .unwrap_or_else(|| surface.agent_type.clone());
        let settings = self.active_settings();
        if !settings.agents.iter().any(|agent| agent.id == agent_type) {
            return Err(NavigationError::new(
                NavigationErrorKind::InvalidConfiguration,
                format!("navigation settings do not define agent type `{agent_type}`"),
            ));
        }
        let half_extent = surface
            .volume_size
            .into_iter()
            .fold(0.0_f32, |largest, value| largest.max(value))
            .max(1.0)
            * 0.5;
        let geometry = collect_bake_geometry(world, surface_entity, &surface, &agent_type);
        let mut diagnostics = bake_geometry_diagnostics(&geometry, surface_entity);
        diagnostics.extend(unsupported_bake_setting_diagnostics(
            &surface,
            surface_entity,
        ));
        let mut asset = if geometry.source_triangles() > 0 {
            self.backend.bake_triangle_mesh(RecastBakeMeshInput {
                agent_type: agent_type.clone(),
                vertices: geometry.vertices.clone(),
                indices: geometry.indices.clone(),
                triangle_areas: geometry.triangle_areas.clone(),
                default_area: surface.default_area,
            })?
        } else if geometry.carved_by_obstacle > 0 || geometry.removed_by_modifier > 0 {
            NavMeshAsset::empty(agent_type.clone())
        } else {
            diagnostics.push(NavMeshBakeDiagnostic {
                severity: NavMeshBakeDiagnosticSeverity::Warning,
                message: "no render mesh or collider bake source was collected; baked surface volume fallback".to_string(),
                entity: surface_entity,
            });
            self.backend.bake_simple_surface(RecastBakeInput {
                agent_type: agent_type.clone(),
                source_vertices: 4,
                source_triangles: 2,
                half_extent,
            })?
        };
        asset.settings_hash = navigation_settings_hash(&surface, &settings);
        asset.area_costs = settings
            .areas
            .iter()
            .map(|area| NavMeshAreaCostAsset {
                area: area.id,
                cost: area.cost,
                walkable: area.walkable,
            })
            .collect();
        let off_mesh_links = if surface.generate_links {
            collect_off_mesh_links(world, &agent_type)
        } else {
            Vec::new()
        };
        if !off_mesh_links.is_empty() {
            diagnostics.push(NavMeshBakeDiagnostic {
                severity: NavMeshBakeDiagnosticSeverity::Info,
                message: format!(
                    "embedded {} active off-mesh link(s) in baked navigation asset",
                    off_mesh_links.len()
                ),
                entity: surface_entity,
            });
            asset.off_mesh_links = off_mesh_links;
        }
        let output_asset = request.output_asset.or(surface.output_asset.clone());
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.stats.active_obstacles = count_obstacles(world);
        state.stats.active_off_mesh_links = count_off_mesh_links(world);
        drop(state);

        Ok(NavMeshBakeReport {
            asset: Some(asset.clone()),
            output_asset,
            surfaces: surfaces.len(),
            source_vertices: geometry.vertices.len(),
            source_triangles: geometry.source_triangles(),
            baked_vertices: asset.vertices.len(),
            baked_polygons: asset.polygons.len(),
            tiles: asset.tiles.len(),
            diagnostics,
        })
    }

    fn load_nav_mesh(&self, asset: NavMeshAsset) -> Result<NavMeshHandle, NavigationError> {
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        let handle = NavMeshHandle(state.next_handle);
        state.next_handle += 1;
        state.loaded.insert(handle, asset);
        state.stats.loaded_nav_meshes = state.loaded.len();
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
        validate_navigation_settings(&settings)?;
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.settings = settings;
        Ok(())
    }

    fn find_path(&self, query: NavPathQuery) -> Result<NavPathResult, NavigationError> {
        let asset = self.selected_asset(query.nav_mesh)?;
        self.backend.find_path(&asset, &query)
    }

    fn sample_position(
        &self,
        query: NavSampleQuery,
    ) -> Result<Option<NavSampleHit>, NavigationError> {
        let asset = self.selected_asset(query.nav_mesh)?;
        self.backend.sample_position(&asset, &query)
    }

    fn raycast(&self, query: NavRaycastQuery) -> Result<NavRaycastResult, NavigationError> {
        let asset = self.selected_asset(query.nav_mesh)?;
        self.backend.raycast(&asset, &query)
    }

    fn tick_world_agents(
        &self,
        world: &mut World,
        dt_seconds: Real,
    ) -> Result<NavAgentTickReport, NavigationError> {
        let mut report = NavAgentTickReport::default();
        if dt_seconds <= 0.0 || !dt_seconds.is_finite() {
            return Ok(report);
        }

        let agents = collect_agents(world);
        report.scanned_agents = agents.len();
        let agent_positions = agent_positions(world, &agents);
        let obstacles = collect_runtime_obstacles(world);
        for (entity, agent) in agents {
            let Some(destination) = agent.destination else {
                continue;
            };
            if !agent.update_position {
                continue;
            }
            let Some(transform) = world.world_transform(entity) else {
                report.blocked_agents += 1;
                report
                    .diagnostics
                    .push(format!("agent {entity} has no world transform"));
                continue;
            };
            let current = transform.translation;
            let destination = Vec3::from_array(destination);
            let movement_target = match self.selected_asset(None) {
                Ok(asset) => match self.backend.find_path_with_obstacles(
                    &asset,
                    &NavPathQuery {
                        nav_mesh: None,
                        start: current.to_array(),
                        end: destination.to_array(),
                        agent_type: agent.agent_type.clone(),
                        area_mask: agent.area_mask,
                    },
                    &recast_carving_obstacles(&obstacles),
                ) {
                    Ok(path) if path.status != NavPathStatus::NoPath => path
                        .points
                        .get(1)
                        .or_else(|| path.points.last())
                        .map(|point| Vec3::from_array(point.position))
                        .unwrap_or(destination),
                    Ok(_) => {
                        report.blocked_agents += 1;
                        report
                            .diagnostics
                            .push(format!("agent {entity} has no path on loaded navmesh"));
                        continue;
                    }
                    Err(error) => {
                        report.blocked_agents += 1;
                        report
                            .diagnostics
                            .push(format!("agent {entity} path query failed: {error}"));
                        continue;
                    }
                },
                Err(_) => destination,
            };
            let movement_target = avoidance_adjusted_target(
                entity,
                current,
                movement_target,
                &agent,
                &obstacles,
                &agent_positions,
            );
            let delta = movement_target - current;
            let distance = delta.length();
            if distance <= agent.stopping_distance {
                continue;
            }
            let step = (agent.speed.max(0.0) * dt_seconds).min(distance);
            let direction = delta.normalize_or_zero();
            let next = current + direction * step;
            let updated = Transform {
                translation: next,
                rotation: if agent.update_rotation && direction.length_squared() > Real::EPSILON {
                    Quat::from_rotation_y(direction.x.atan2(-direction.z))
                } else {
                    transform.rotation
                },
                ..transform
            };
            match world.update_transform(entity, updated) {
                Ok(true) => report.moved_agents += 1,
                Ok(false) => {}
                Err(error) => {
                    report.blocked_agents += 1;
                    report
                        .diagnostics
                        .push(format!("agent {entity} could not move: {error}"));
                }
            }
        }
        let mut state = self.state.lock().expect("navigation state lock poisoned");
        state.stats.active_agents = report.scanned_agents;
        state.stats.active_obstacles = obstacles.len();
        state.stats.active_off_mesh_links = count_off_mesh_links(world);
        Ok(report)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        let state = self.state.lock().expect("navigation state lock poisoned");
        state.stats.clone()
    }
}

#[derive(Debug)]
struct NavigationRuntimeState {
    next_handle: u64,
    loaded: HashMap<NavMeshHandle, NavMeshAsset>,
    settings: NavigationSettingsAsset,
    stats: NavigationRuntimeStats,
}

impl Default for NavigationRuntimeState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
            stats: NavigationRuntimeStats::default(),
        }
    }
}

fn collect_surfaces(world: &World) -> Vec<(u64, NavMeshSurfaceDescriptor)> {
    world
        .nodes()
        .iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_SURFACE_COMPONENT_TYPE)?;
            let surface = parse_component::<NavMeshSurfaceDescriptor>(value);
            surface.enabled.then_some((node.id, surface))
        })
        .collect()
}

fn collect_agents(world: &World) -> Vec<(u64, NavMeshAgentDescriptor)> {
    world
        .nodes()
        .iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)?;
            Some((node.id, parse_component::<NavMeshAgentDescriptor>(value)))
        })
        .collect()
}

#[derive(Clone, Debug, Default)]
struct BakeGeometry {
    vertices: Vec<[Real; 3]>,
    indices: Vec<u32>,
    triangle_areas: Vec<u8>,
    source_entities: usize,
    skipped_navigation_components: usize,
    removed_by_modifier: usize,
    modified_by_area_override: usize,
    carved_by_obstacle: usize,
}

impl BakeGeometry {
    fn source_triangles(&self) -> usize {
        self.indices.len() / 3
    }

    fn push_quad_from_matrix(&mut self, matrix: Mat4, half_extents: Vec3, area: u8) {
        let top_y = half_extents.y.max(0.0);
        let corners = [
            Vec3::new(-half_extents.x, top_y, -half_extents.z),
            Vec3::new(half_extents.x, top_y, -half_extents.z),
            Vec3::new(half_extents.x, top_y, half_extents.z),
            Vec3::new(-half_extents.x, top_y, half_extents.z),
        ];
        let base = self.vertices.len() as u32;
        self.vertices.extend(
            corners
                .into_iter()
                .map(|corner| matrix.transform_point3(corner).to_array()),
        );
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.triangle_areas.extend_from_slice(&[area, area]);
    }

    fn push_disc_from_matrix(&mut self, matrix: Mat4, radius: Real, local_y: Real, area: u8) {
        const SEGMENTS: u32 = 12;
        let radius = radius.max(0.05);
        let center = self.vertices.len() as u32;
        self.vertices.push(
            matrix
                .transform_point3(Vec3::new(0.0, local_y, 0.0))
                .to_array(),
        );
        for index in 0..SEGMENTS {
            let angle = (index as Real / SEGMENTS as Real) * std::f32::consts::TAU;
            let point = Vec3::new(angle.cos() * radius, local_y, angle.sin() * radius);
            self.vertices
                .push(matrix.transform_point3(point).to_array());
        }
        for index in 0..SEGMENTS {
            let next = if index + 1 == SEGMENTS {
                center + 1
            } else {
                center + index + 2
            };
            self.indices
                .extend_from_slice(&[center, center + index + 1, next]);
            self.triangle_areas.push(area);
        }
    }
}

fn collect_bake_geometry(
    world: &World,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
    agent_type: &str,
) -> BakeGeometry {
    let mut geometry = BakeGeometry::default();
    let carved_obstacles = collect_runtime_obstacles(world)
        .into_iter()
        .filter(|obstacle| obstacle.carve)
        .collect::<Vec<_>>();
    let surface_area_override = surface_entity
        .and_then(|entity| direct_modifier(world, entity, agent_type))
        .filter(|modifier| modifier.override_area)
        .map(|modifier| modifier.area);
    for node in world.nodes() {
        if should_exclude_from_bake(world, node.id) {
            geometry.skipped_navigation_components += 1;
            continue;
        }
        if !node_matches_surface_collection(world, node, surface_entity, surface) {
            continue;
        }

        let modifier = effective_modifier(world, node.id, agent_type);
        if matches!(
            modifier.as_ref().map(|modifier| modifier.mode),
            Some(NavMeshModifierMode::Remove)
        ) {
            geometry.removed_by_modifier += 1;
            continue;
        }
        if node_intersects_obstacle(world, node, &carved_obstacles) {
            geometry.carved_by_obstacle += 1;
            continue;
        }
        let area_override = modifier
            .as_ref()
            .filter(|modifier| modifier.override_area)
            .map(|modifier| modifier.area)
            .or(surface_area_override);
        let area = area_override.unwrap_or(surface.default_area);
        if area_override.is_some() {
            geometry.modified_by_area_override += 1;
        }

        let before = geometry.source_triangles();
        match surface.use_geometry {
            NavMeshUseGeometry::RenderMeshes => {
                collect_render_node_geometry(world, node, &mut geometry, area)
            }
            NavMeshUseGeometry::PhysicsColliders => {
                collect_collider_geometry(world, node, &mut geometry, area)
            }
        }
        if geometry.source_triangles() > before {
            geometry.source_entities += 1;
        }
    }
    geometry
}

fn collect_render_node_geometry(
    world: &World,
    node: &SceneNode,
    geometry: &mut BakeGeometry,
    area: u8,
) {
    if node.mesh.is_none() && !matches!(node.kind, NodeKind::Cube | NodeKind::Mesh) {
        return;
    }
    let Some(transform) = world.world_transform(node.id) else {
        return;
    };
    geometry.push_quad_from_matrix(transform.matrix(), Vec3::splat(0.5), area);
}

fn collect_collider_geometry(
    world: &World,
    node: &SceneNode,
    geometry: &mut BakeGeometry,
    area: u8,
) {
    let Some(collider) = node.collider.as_ref() else {
        return;
    };
    if collider.sensor {
        return;
    }
    let Some(transform) = world.world_transform(node.id) else {
        return;
    };
    let matrix = transform.matrix() * collider.local_transform.matrix();
    match &collider.shape {
        ColliderShape::Box { half_extents } => {
            geometry.push_quad_from_matrix(matrix, *half_extents, area);
        }
        ColliderShape::Sphere { radius } => {
            geometry.push_disc_from_matrix(matrix, *radius, 0.0, area);
        }
        ColliderShape::Capsule {
            radius,
            half_height,
        } => {
            geometry.push_disc_from_matrix(matrix, *radius, *half_height, area);
        }
    }
}

fn should_exclude_from_bake(world: &World, entity: u64) -> bool {
    world
        .dynamic_component(entity, NAV_MESH_SURFACE_COMPONENT_TYPE)
        .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
        || world
            .dynamic_component(entity, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
            .is_some()
}

fn node_matches_surface_collection(
    world: &World,
    node: &SceneNode,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
) -> bool {
    if world.active_in_hierarchy(node.id) == Some(false) {
        return false;
    }
    if !layer_included(world, node, surface) {
        return false;
    }
    match surface.collect_mode {
        zircon_runtime::core::framework::navigation::NavMeshCollectMode::AllObjects => true,
        zircon_runtime::core::framework::navigation::NavMeshCollectMode::Hierarchy => {
            surface_entity
                .is_some_and(|root| node.id == root || is_descendant_of(world, node.id, root))
        }
        zircon_runtime::core::framework::navigation::NavMeshCollectMode::Volume => {
            node_inside_surface_volume(world, node, surface_entity, surface)
        }
        zircon_runtime::core::framework::navigation::NavMeshCollectMode::ModifierOnly => {
            effective_modifier(world, node.id, &surface.agent_type).is_some()
        }
    }
}

fn layer_included(world: &World, node: &SceneNode, surface: &NavMeshSurfaceDescriptor) -> bool {
    if surface.include_layers.is_empty() {
        return true;
    }
    let mask = world.render_layer_mask(node.id).unwrap_or_default();
    surface.include_layers.iter().any(|layer| {
        layer.eq_ignore_ascii_case(&node.name)
            || layer.eq_ignore_ascii_case(node_kind_name(&node.kind))
            || layer
                .strip_prefix("layer:")
                .and_then(|value| value.parse::<u32>().ok())
                .is_some_and(|bit| bit < 32 && (mask & (1_u32 << bit)) != 0)
    })
}

fn node_kind_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "camera",
        NodeKind::Cube => "cube",
        NodeKind::Mesh => "mesh",
        NodeKind::DirectionalLight => "directional_light",
        NodeKind::PointLight => "point_light",
        NodeKind::SpotLight => "spot_light",
    }
}

fn is_descendant_of(world: &World, entity: u64, ancestor: u64) -> bool {
    let mut current = world.parent_of(entity);
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = world.parent_of(parent);
    }
    false
}

fn node_inside_surface_volume(
    world: &World,
    node: &SceneNode,
    surface_entity: Option<u64>,
    surface: &NavMeshSurfaceDescriptor,
) -> bool {
    let center = surface_entity
        .and_then(|entity| world.world_transform(entity))
        .map(|transform| {
            transform
                .matrix()
                .transform_point3(Vec3::from_array(surface.volume_center))
        })
        .unwrap_or_else(|| Vec3::from_array(surface.volume_center));
    let half_size = Vec3::from_array(surface.volume_size).abs() * 0.5;
    let position = world
        .world_transform(node.id)
        .map(|transform| transform.translation)
        .unwrap_or(node.transform.translation);
    let delta = (position - center).abs();
    delta.x <= half_size.x && delta.y <= half_size.y && delta.z <= half_size.z
}

fn effective_modifier(
    world: &World,
    entity: u64,
    agent_type: &str,
) -> Option<NavMeshModifierDescriptor> {
    if let Some(modifier) = direct_modifier(world, entity, agent_type) {
        return Some(modifier);
    }

    let mut current = world.parent_of(entity);
    while let Some(parent) = current {
        if let Some(modifier) = direct_modifier(world, parent, agent_type) {
            if modifier.apply_to_children {
                return Some(modifier);
            }
        }
        current = world.parent_of(parent);
    }
    None
}

fn direct_modifier(
    world: &World,
    entity: u64,
    agent_type: &str,
) -> Option<NavMeshModifierDescriptor> {
    let value = world.dynamic_component(entity, NAV_MESH_MODIFIER_COMPONENT_TYPE)?;
    let modifier = parse_component::<NavMeshModifierDescriptor>(value);
    modifier_affects_agent(&modifier, agent_type).then_some(modifier)
}

fn modifier_affects_agent(modifier: &NavMeshModifierDescriptor, agent_type: &str) -> bool {
    modifier.affected_agents.is_empty()
        || modifier
            .affected_agents
            .iter()
            .any(|affected| affected == agent_type)
}

fn collect_off_mesh_links(world: &World, agent_type: &str) -> Vec<NavMeshLinkAsset> {
    world
        .nodes()
        .iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)?;
            let link = parse_component::<NavMeshOffMeshLinkDescriptor>(value);
            if !link.activated || link.agent_type != agent_type {
                return None;
            }
            Some(NavMeshLinkAsset {
                start: link_endpoint_world_position(
                    world,
                    node.id,
                    link.start_entity,
                    link.start_local_point,
                ),
                end: link_endpoint_world_position(
                    world,
                    node.id,
                    link.end_entity,
                    link.end_local_point,
                ),
                width: link.width,
                bidirectional: link.bidirectional,
                area: link.area_type,
                cost_override: link.cost_override,
                traversal_mode: link.traversal_mode,
            })
        })
        .collect()
}

fn link_endpoint_world_position(
    world: &World,
    owner: u64,
    endpoint_entity: Option<u64>,
    local_point: [Real; 3],
) -> [Real; 3] {
    let transform_entity = endpoint_entity.unwrap_or(owner);
    world
        .world_transform(transform_entity)
        .unwrap_or_default()
        .matrix()
        .transform_point3(Vec3::from_array(local_point))
        .to_array()
}

fn bake_geometry_diagnostics(
    geometry: &BakeGeometry,
    surface_entity: Option<u64>,
) -> Vec<NavMeshBakeDiagnostic> {
    let mut diagnostics = vec![NavMeshBakeDiagnostic {
        severity: NavMeshBakeDiagnosticSeverity::Info,
        message: format!(
            "collected {} navigation bake source entity/entities ({} triangle(s))",
            geometry.source_entities,
            geometry.source_triangles()
        ),
        entity: surface_entity,
    }];
    if geometry.skipped_navigation_components > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "excluded {} navigation authoring/runtime component node(s) from bake geometry",
                geometry.skipped_navigation_components
            ),
            entity: surface_entity,
        });
    }
    if geometry.removed_by_modifier > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "removed {} bake source node(s) by NavMeshModifier",
                geometry.removed_by_modifier
            ),
            entity: surface_entity,
        });
    }
    if geometry.modified_by_area_override > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "applied area override to {} bake source node(s)",
                geometry.modified_by_area_override
            ),
            entity: surface_entity,
        });
    }
    if geometry.carved_by_obstacle > 0 {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Info,
            message: format!(
                "carved {} bake source node(s) by stationary NavMeshObstacle",
                geometry.carved_by_obstacle
            ),
            entity: surface_entity,
        });
    }
    diagnostics
}

fn unsupported_bake_setting_diagnostics(
    surface: &NavMeshSurfaceDescriptor,
    surface_entity: Option<u64>,
) -> Vec<NavMeshBakeDiagnostic> {
    let mut diagnostics = Vec::new();
    if surface.override_voxel_size.is_some()
        || surface.override_tile_size.is_some()
        || surface.min_region_area != NavMeshSurfaceDescriptor::default().min_region_area
        || surface.build_height_mesh
    {
        diagnostics.push(NavMeshBakeDiagnostic {
            severity: NavMeshBakeDiagnosticSeverity::Warning,
            message: "advanced Recast bake knobs are recorded in the settings hash but the v1 fallback backend does not yet rasterize voxels, tiles, regions, or height meshes".to_string(),
            entity: surface_entity,
        });
    }
    diagnostics
}

fn count_obstacles(world: &World) -> usize {
    world
        .nodes()
        .iter()
        .filter(|node| {
            world
                .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
                .is_some()
        })
        .count()
}

fn count_off_mesh_links(world: &World) -> usize {
    world
        .nodes()
        .iter()
        .filter(|node| {
            world
                .dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
                .is_some()
        })
        .count()
}

fn agent_positions(
    world: &World,
    agents: &[(u64, NavMeshAgentDescriptor)],
) -> Vec<(u64, Vec3, Real)> {
    agents
        .iter()
        .filter_map(|(entity, agent)| {
            world
                .world_transform(*entity)
                .map(|transform| (*entity, transform.translation, agent.radius.max(0.05)))
        })
        .collect()
}

fn avoidance_adjusted_target(
    entity: u64,
    current: Vec3,
    target: Vec3,
    agent: &NavMeshAgentDescriptor,
    obstacles: &[RuntimeObstacle],
    agents: &[(u64, Vec3, Real)],
) -> Vec3 {
    if matches!(
        agent.avoidance_quality,
        zircon_runtime::core::framework::navigation::NavAvoidanceQuality::None
    ) {
        return target;
    }
    let mut avoidance = Vec3::ZERO;
    for obstacle in obstacles
        .iter()
        .filter(|obstacle| obstacle.avoidance_enabled)
    {
        let away = current - obstacle.center;
        let distance = distance_xz(current, obstacle.center);
        let limit = obstacle.radius + agent.radius.max(0.05) + 0.5;
        if distance > 0.001 && distance < limit {
            avoidance += Vec3::new(away.x, 0.0, away.z).normalize_or_zero() * (limit - distance);
        }
    }
    for (other_entity, other_position, other_radius) in agents {
        if *other_entity == entity {
            continue;
        }
        let away = current - *other_position;
        let distance = distance_xz(current, *other_position);
        let limit = agent.radius.max(0.05) + *other_radius + 0.25;
        if distance > 0.001 && distance < limit {
            avoidance += Vec3::new(away.x, 0.0, away.z).normalize_or_zero() * (limit - distance);
        }
    }
    target + avoidance
}

pub fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    let mut stats = NavigationRuntimeStats::default();
    for node in world.nodes() {
        if world
            .dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_agents += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_obstacles += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_off_mesh_links += 1;
        }
    }
    stats
}

pub fn default_agent_type() -> &'static str {
    DEFAULT_AGENT_TYPE
}
