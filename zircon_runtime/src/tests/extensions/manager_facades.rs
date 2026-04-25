use crate::core::CoreRuntime;
use std::thread;
use std::time::Duration;

#[test]
fn physics_and_animation_runtime_extensions_keep_manager_handles_under_core_manager_facades() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let physics_mod_source =
        std::fs::read_to_string(runtime_root.join("src/physics/mod.rs")).unwrap_or_default();
    let physics_service_source =
        std::fs::read_to_string(runtime_root.join("src/physics/service_types.rs"))
            .unwrap_or_default();
    let animation_mod_source =
        std::fs::read_to_string(runtime_root.join("src/animation/mod.rs")).unwrap_or_default();
    let animation_service_source =
        std::fs::read_to_string(runtime_root.join("src/animation/service_types.rs"))
            .unwrap_or_default();
    let net_mod_source =
        std::fs::read_to_string(runtime_root.join("src/extensions/net/mod.rs")).unwrap_or_default();
    let net_service_source =
        std::fs::read_to_string(runtime_root.join("src/extensions/net/service_types.rs"))
            .unwrap_or_default();
    let sound_mod_source =
        std::fs::read_to_string(runtime_root.join("src/extensions/sound/mod.rs"))
            .unwrap_or_default();
    let sound_service_source =
        std::fs::read_to_string(runtime_root.join("src/extensions/sound/service_types.rs"))
            .unwrap_or_default();
    let manager_mod_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/mod.rs")).unwrap_or_default();
    let manager_resolver_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/resolver.rs"))
            .unwrap_or_default();
    let manager_service_names_source =
        std::fs::read_to_string(runtime_root.join("src/core/manager/service_names.rs"))
            .unwrap_or_default();

    for required in [
        "DefaultPhysicsManager",
        "impl crate::core::framework::physics::PhysicsManager for DefaultPhysicsManager",
        "PHYSICS_MANAGER_NAME",
    ] {
        assert!(
            physics_mod_source.contains(required) || physics_service_source.contains(required),
            "runtime physics subtree should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultAnimationManager",
        "impl crate::core::framework::animation::AnimationManager for DefaultAnimationManager",
        "ANIMATION_MANAGER_NAME",
    ] {
        assert!(
            animation_mod_source.contains(required) || animation_service_source.contains(required),
            "runtime animation subtree should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultNetManager",
        "impl crate::core::framework::net::NetManager for DefaultNetManager",
        "NET_MANAGER_NAME",
    ] {
        assert!(
            net_mod_source.contains(required) || net_service_source.contains(required),
            "runtime net subtree should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "DefaultSoundManager",
        "impl crate::core::framework::sound::SoundManager for DefaultSoundManager",
        "SOUND_MANAGER_NAME",
    ] {
        assert!(
            sound_mod_source.contains(required) || sound_service_source.contains(required),
            "runtime sound subtree should keep its framework-backed manager service wiring `{required}`"
        );
    }

    for required in [
        "PhysicsManagerHandle",
        "AnimationManagerHandle",
        "NetManagerHandle",
        "SoundManagerHandle",
        "resolve_physics_manager",
        "resolve_animation_manager",
        "resolve_net_manager",
        "resolve_sound_manager",
    ] {
        assert!(
            manager_mod_source.contains(required) || manager_resolver_source.contains(required),
            "core manager surface should own public physics/animation/net manager handle wiring `{required}`"
        );
    }

    for required in [
        "PHYSICS_MANAGER_NAME",
        "ANIMATION_MANAGER_NAME",
        "NET_MANAGER_NAME",
        "SOUND_MANAGER_NAME",
    ] {
        assert!(
            manager_mod_source.contains(required)
                || manager_resolver_source.contains(required)
                || manager_service_names_source.contains(required),
            "core manager surface should own public physics/animation/net/sound manager service names `{required}`"
        );
    }
}

#[test]
fn physics_and_animation_managers_resolve_through_framework_facades() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::physics::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::animation::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::physics::PHYSICS_MODULE_NAME)
        .unwrap();
    runtime
        .activate_module(crate::animation::ANIMATION_MODULE_NAME)
        .unwrap();

    let physics = crate::core::manager::resolve_physics_manager(&runtime.handle()).unwrap();
    let animation = crate::core::manager::resolve_animation_manager(&runtime.handle()).unwrap();

    assert_eq!(physics.backend_name(), "unconfigured");
    assert_eq!(physics.settings().fixed_hz, 60);
    assert!(animation.playback_settings().enabled);
    assert!(animation.playback_settings().property_tracks);

    let track_path = crate::core::framework::animation::AnimationTrackPath::parse(
        "root/child:transform.translation",
    )
    .unwrap();
    assert_eq!(animation.normalize_track_path(&track_path), track_path);
}

#[test]
fn net_manager_resolves_through_framework_facade_and_roundtrips_udp_loopback_packets() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::extensions::net::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::extensions::net::NET_MODULE_NAME)
        .unwrap();

    let net = crate::core::manager::resolve_net_manager(&runtime.handle()).unwrap();
    let left = net
        .bind_udp(&crate::core::framework::net::NetEndpoint::new(
            "127.0.0.1",
            0,
        ))
        .unwrap();
    let right = net
        .bind_udp(&crate::core::framework::net::NetEndpoint::new(
            "127.0.0.1",
            0,
        ))
        .unwrap();
    let left_endpoint = net.local_endpoint(left).unwrap();
    let right_endpoint = net.local_endpoint(right).unwrap();

    assert!(left_endpoint.port > 0);
    assert!(right_endpoint.port > 0);

    net.send_udp(left, &right_endpoint, b"ping").unwrap();

    let mut packets = Vec::new();
    for _ in 0..20 {
        packets = net.poll_udp(right, 8).unwrap();
        if !packets.is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].source, left_endpoint);
    assert_eq!(packets[0].payload, b"ping");

    net.close_socket(left).unwrap();
    net.close_socket(right).unwrap();
    assert_eq!(
        net.close_socket(left),
        Err(crate::core::framework::net::NetError::UnknownSocket { socket: left })
    );
}

#[test]
fn net_manager_poll_udp_respects_packet_budget_and_leaves_remaining_packets() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::extensions::net::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::extensions::net::NET_MODULE_NAME)
        .unwrap();

    let net = crate::core::manager::resolve_net_manager(&runtime.handle()).unwrap();
    let sender = net
        .bind_udp(&crate::core::framework::net::NetEndpoint::new(
            "127.0.0.1",
            0,
        ))
        .unwrap();
    let receiver = net
        .bind_udp(&crate::core::framework::net::NetEndpoint::new(
            "127.0.0.1",
            0,
        ))
        .unwrap();
    let receiver_endpoint = net.local_endpoint(receiver).unwrap();

    net.send_udp(sender, &receiver_endpoint, b"first").unwrap();
    net.send_udp(sender, &receiver_endpoint, b"second").unwrap();

    let first_poll = wait_for_packet_count(&*net, receiver, 1);
    assert_eq!(first_poll.len(), 1);
    assert_eq!(first_poll[0].payload, b"first");

    let second_poll = wait_for_packet_count(&*net, receiver, 1);
    assert_eq!(second_poll.len(), 1);
    assert_eq!(second_poll[0].payload, b"second");

    net.close_socket(sender).unwrap();
    net.close_socket(receiver).unwrap();
}

fn wait_for_packet_count(
    net: &dyn crate::core::framework::net::NetManager,
    socket: crate::core::framework::net::NetSocketId,
    max_packets: usize,
) -> Vec<crate::core::framework::net::NetPacket> {
    let mut packets = Vec::new();
    for _ in 0..20 {
        packets = net.poll_udp(socket, max_packets).unwrap();
        if packets.len() == max_packets {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    packets
}
