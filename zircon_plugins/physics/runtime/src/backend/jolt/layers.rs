use std::ffi::{c_uint, c_void};

use joltc_sys::{
    JPC_BroadPhaseLayer, JPC_BroadPhaseLayerInterfaceFns, JPC_ObjectLayer,
    JPC_ObjectLayerPairFilterFns, JPC_ObjectVsBroadPhaseLayerFilterFns,
};

pub(super) const OBJECT_LAYER_NON_MOVING: JPC_ObjectLayer = 0;
pub(super) const OBJECT_LAYER_MOVING: JPC_ObjectLayer = 1;

const BROAD_PHASE_LAYER_NON_MOVING: JPC_BroadPhaseLayer = 0;
const BROAD_PHASE_LAYER_MOVING: JPC_BroadPhaseLayer = 1;
const BROAD_PHASE_LAYER_COUNT: JPC_BroadPhaseLayer = 2;

pub(super) const BROAD_PHASE_LAYER_INTERFACE: JPC_BroadPhaseLayerInterfaceFns =
    JPC_BroadPhaseLayerInterfaceFns {
        GetNumBroadPhaseLayers: Some(get_num_broad_phase_layers as _),
        GetBroadPhaseLayer: Some(get_broad_phase_layer as _),
    };

pub(super) const OBJECT_VS_BROAD_PHASE_FILTER: JPC_ObjectVsBroadPhaseLayerFilterFns =
    JPC_ObjectVsBroadPhaseLayerFilterFns {
        ShouldCollide: Some(object_vs_broad_phase_should_collide as _),
    };

pub(super) const OBJECT_LAYER_PAIR_FILTER: JPC_ObjectLayerPairFilterFns =
    JPC_ObjectLayerPairFilterFns {
        ShouldCollide: Some(object_layer_pair_should_collide as _),
    };

unsafe extern "C" fn get_num_broad_phase_layers(_state: *const c_void) -> c_uint {
    BROAD_PHASE_LAYER_COUNT.into()
}

unsafe extern "C" fn get_broad_phase_layer(
    _state: *const c_void,
    layer: JPC_ObjectLayer,
) -> JPC_BroadPhaseLayer {
    match layer {
        OBJECT_LAYER_NON_MOVING => BROAD_PHASE_LAYER_NON_MOVING,
        OBJECT_LAYER_MOVING => BROAD_PHASE_LAYER_MOVING,
        _ => BROAD_PHASE_LAYER_MOVING,
    }
}

unsafe extern "C" fn object_vs_broad_phase_should_collide(
    _state: *const c_void,
    object_layer: JPC_ObjectLayer,
    broad_phase_layer: JPC_BroadPhaseLayer,
) -> bool {
    match object_layer {
        OBJECT_LAYER_NON_MOVING => broad_phase_layer == BROAD_PHASE_LAYER_MOVING,
        OBJECT_LAYER_MOVING => true,
        _ => false,
    }
}

unsafe extern "C" fn object_layer_pair_should_collide(
    _state: *const c_void,
    left: JPC_ObjectLayer,
    right: JPC_ObjectLayer,
) -> bool {
    match left {
        OBJECT_LAYER_NON_MOVING => right == OBJECT_LAYER_MOVING,
        OBJECT_LAYER_MOVING => matches!(right, OBJECT_LAYER_NON_MOVING | OBJECT_LAYER_MOVING),
        _ => false,
    }
}
