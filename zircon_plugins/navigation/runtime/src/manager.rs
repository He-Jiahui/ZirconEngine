use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;
use zircon_plugin_navigation_recast::{RecastBackend, RecastBakeInput, RecastBakeMeshInput};
use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshAgentDescriptor, NavMeshBakeDiagnostic,
    NavMeshBakeDiagnosticSeverity, NavMeshBakeReport, NavMeshBakeRequest, NavMeshHandle,
    NavMeshModifierDescriptor, NavMeshModifierMode, NavMeshOffMeshLinkDescriptor,
    NavMeshSurfaceDescriptor, NavMeshUseGeometry, NavPathQuery, NavPathResult, NavRaycastQuery,
    NavRaycastResult, NavSampleHit, NavSampleQuery, NavigationError, NavigationManager,
    NavigationRuntimeStats, DEFAULT_AGENT_TYPE, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_MODIFIER_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Mat4, Real, Transform, Vec3};
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
            .or_else(|| state.loaded.keys().copied().next())
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
        let half_extent = surface
            .volume_size
            .into_iter()
            .fold(0.0_f32, |largest, value| largest.max(value))
            .max(1.0)
            * 0.5;
        let geometry = collect_bake_geometry(world, surface_entity, &surface, &agent_type);
        let mut diagnostics = bake_geometry_diagnostics(&geometry, surface_entity);
        let mut asset = if geometry.source_triangles() > 0 {
            self.backend.bake_triangle_mesh(RecastBakeMeshInput {
                agent_type: agent_type.clone(),
                vertices: geometry.vertices.clone(),
                indices: geometry.indices.clone(),
                triangle_areas: geometry.triangle_areas.clone(),
                default_area: surface.default_area,
            })?
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
        let off_mesh_links = collect_off_mesh_links(world, &agent_type);
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
        Ok(handle)
    }

    fn load_navigation_settings(
        &self,
        settings: NavigationSettingsAsset,
    ) -> Result<(), NavigationError> {
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
            let delta = destination - current;
            let distance = delta.length();
            if distance <= agent.stopping_distance {
                continue;
            }
            let step = (agent.speed.max(0.0) * dt_seconds).min(distance);
            let next = current + delta.normalize_or_zero() * step;
            let updated = Transform {
                translation: next,
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
        Ok(report)
    }

    fn stats(&self) -> NavigationRuntimeStats {
        let state = self.state.lock().expect("navigation state lock poisoned");
        NavigationRuntimeStats {
            loaded_nav_meshes: state.loaded.len(),
            active_agents: 0,
            active_obstacles: 0,
            active_off_mesh_links: 0,
        }
    }
}

#[derive(Debug)]
struct NavigationRuntimeState {
    next_handle: u64,
    loaded: HashMap<NavMeshHandle, NavMeshAsset>,
    settings: NavigationSettingsAsset,
}

impl Default for NavigationRuntimeState {
    fn default() -> Self {
        Self {
            next_handle: 1,
            loaded: HashMap::new(),
            settings: NavigationSettingsAsset::default(),
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
        let area = modifier
            .as_ref()
            .filter(|modifier| modifier.override_area)
            .map(|modifier| modifier.area)
            .unwrap_or(surface.default_area);
        if modifier
            .as_ref()
            .is_some_and(|modifier| modifier.override_area)
        {
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
                "excluded {} navigation runtime component node(s) from bake geometry",
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
    diagnostics
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

fn parse_component<T>(value: &Value) -> T
where
    T: Default + serde::de::DeserializeOwned,
{
    serde_json::from_value(normalize_scene_property_json(value)).unwrap_or_default()
}

fn normalize_scene_property_json(value: &Value) -> Value {
    match value {
        Value::Object(object) if object.len() == 1 && object.contains_key("resource") => object
            .get("resource")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
        Value::Object(object) if object.len() == 1 && object.contains_key("entity") => {
            object.get("entity").cloned().unwrap_or(Value::Null)
        }
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| (key.clone(), normalize_scene_property_json(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(normalize_scene_property_json)
                .collect::<Vec<_>>(),
        ),
        value => value.clone(),
    }
}

pub fn default_agent_type() -> &'static str {
    DEFAULT_AGENT_TYPE
}
