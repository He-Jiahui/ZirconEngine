use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr::NonNull;

use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavMeshAgentDescriptor, NavigationError, NavigationErrorKind,
};
use zircon_runtime::core::math::Real;

use crate::asset_ffi::detour_area_costs;
use crate::detour::DetourQuery;
use crate::ffi::{
    self, ZrNavCrowdAgentParams, ZrNavCrowdAgentState, ZrNavCrowdCommandResult,
    ZrNavCrowdCreateResult,
};

const ZR_NAV_CROWD_OK: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecastCrowdAgentHandle(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecastCrowdAgentState {
    pub handle: RecastCrowdAgentHandle,
    pub traversal_state: u8,
    pub target_state: u8,
    pub partial_path: bool,
    pub position: [Real; 3],
    pub desired_velocity: [Real; 3],
    pub avoidance_velocity: [Real; 3],
    pub velocity: [Real; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecastCrowdConfig {
    pub max_agents: u32,
    pub max_agent_radius: Real,
}

impl Default for RecastCrowdConfig {
    fn default() -> Self {
        Self {
            max_agents: 64,
            max_agent_radius: 1.0,
        }
    }
}

#[derive(Debug)]
pub struct RecastCrowd {
    handle: NonNull<c_void>,
    capacity: usize,
}

unsafe impl Send for RecastCrowd {}

impl RecastCrowd {
    pub fn from_asset(
        asset: &NavMeshAsset,
        config: RecastCrowdConfig,
    ) -> Result<Self, NavigationError> {
        if config.max_agents == 0
            || !config.max_agent_radius.is_finite()
            || config.max_agent_radius <= 0.0
        {
            return Err(crowd_error(
                "crowd capacity and maximum radius must be positive",
            ));
        }
        let query = DetourQuery::from_asset(asset)
            .ok_or_else(|| crowd_error("navmesh asset could not create a Detour query"))?;
        let area_costs = detour_area_costs(asset);
        let mut result = ZrNavCrowdCreateResult::default();
        unsafe {
            ffi::zr_nav_crowd_create(
                query.as_raw(),
                config.max_agents,
                config.max_agent_radius,
                area_costs.as_ptr(),
                area_costs.len() as u32,
                &mut result,
            );
        }
        if result.status != ZR_NAV_CROWD_OK {
            return Err(crowd_error(native_message(&result.message)));
        }
        let handle = NonNull::new(result.crowd)
            .ok_or_else(|| crowd_error("native crowd returned a null handle"))?;
        query.into_raw();
        Ok(Self {
            handle,
            capacity: result.capacity as usize,
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn add_agent(
        &mut self,
        position: [Real; 3],
        agent: &NavMeshAgentDescriptor,
    ) -> Result<RecastCrowdAgentHandle, NavigationError> {
        let radius = agent.radius.max(0.01);
        let params = ZrNavCrowdAgentParams {
            radius,
            height: agent.height.max(0.01),
            max_acceleration: agent.acceleration.max(0.0),
            max_speed: agent.speed.max(0.0),
            collision_query_range: radius * 8.0,
            path_optimization_range: radius * 30.0,
            separation_weight: separation_weight(agent.priority),
            avoidance_quality: avoidance_quality_index(agent.avoidance_quality),
            avoidance_priority: agent.priority,
            area_mask: agent.area_mask,
        };
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_add_agent(
                self.handle.as_ptr(),
                position.as_ptr(),
                &params,
                &mut result,
            );
        }
        command_result(&result)?;
        Ok(RecastCrowdAgentHandle(result.agent_id))
    }

    pub fn remove_agent(&mut self, handle: RecastCrowdAgentHandle) -> Result<(), NavigationError> {
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_remove_agent(self.handle.as_ptr(), handle.0, &mut result);
        }
        command_result(&result)
    }

    pub fn set_target(
        &mut self,
        handle: RecastCrowdAgentHandle,
        target: [Real; 3],
    ) -> Result<(), NavigationError> {
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_set_target(
                self.handle.as_ptr(),
                handle.0,
                target.as_ptr(),
                &mut result,
            );
        }
        command_result(&result)
    }

    pub fn clear_target(&mut self, handle: RecastCrowdAgentHandle) -> Result<(), NavigationError> {
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_set_target(
                self.handle.as_ptr(),
                handle.0,
                std::ptr::null(),
                &mut result,
            );
        }
        command_result(&result)
    }

    pub fn sync_agent_position(
        &mut self,
        handle: RecastCrowdAgentHandle,
        position: [Real; 3],
    ) -> Result<(), NavigationError> {
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_sync_agent_position(
                self.handle.as_ptr(),
                handle.0,
                position.as_ptr(),
                &mut result,
            );
        }
        command_result(&result)
    }

    pub fn update(&mut self, dt_seconds: Real) -> Result<(), NavigationError> {
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_update(self.handle.as_ptr(), dt_seconds, &mut result);
        }
        command_result(&result)
    }

    pub fn read_states(&self) -> Result<Vec<RecastCrowdAgentState>, NavigationError> {
        let mut states = vec![ZrNavCrowdAgentState::default(); self.capacity];
        let mut result = ZrNavCrowdCommandResult::default();
        unsafe {
            ffi::zr_nav_crowd_read_states(
                self.handle.as_ptr(),
                states.as_mut_ptr(),
                states.len() as u32,
                &mut result,
            );
        }
        command_result(&result)?;
        states.truncate(result.state_count as usize);
        Ok(states
            .into_iter()
            .filter(|state| state.active != 0)
            .map(|state| RecastCrowdAgentState {
                handle: RecastCrowdAgentHandle(state.agent_id),
                traversal_state: state.traversal_state,
                target_state: state.target_state,
                partial_path: state.partial_path != 0,
                position: state.position,
                desired_velocity: state.desired_velocity,
                avoidance_velocity: state.avoidance_velocity,
                velocity: state.velocity,
            })
            .collect())
    }
}

impl Drop for RecastCrowd {
    fn drop(&mut self) {
        unsafe {
            ffi::zr_nav_crowd_free(self.handle.as_ptr());
        }
    }
}

fn command_result(result: &ZrNavCrowdCommandResult) -> Result<(), NavigationError> {
    if result.status == ZR_NAV_CROWD_OK {
        Ok(())
    } else {
        Err(crowd_error(native_message(&result.message)))
    }
}

fn avoidance_quality_index(quality: NavAvoidanceQuality) -> u8 {
    match quality {
        NavAvoidanceQuality::None => 0,
        NavAvoidanceQuality::Low => 1,
        NavAvoidanceQuality::Medium => 2,
        NavAvoidanceQuality::High => 3,
    }
}

fn separation_weight(priority: u8) -> Real {
    0.5 + (255.0 - Real::from(priority)) / 255.0 * 1.5
}

fn native_message(message: &[c_char; 256]) -> String {
    unsafe { CStr::from_ptr(message.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn crowd_error(message: impl Into<String>) -> NavigationError {
    NavigationError::new(NavigationErrorKind::BackendFailure, message)
}
