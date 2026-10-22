use super::*;

#[test]
fn visible_duration_stops_when_a_user_hides_the_card_before_readiness() {
    let start = Instant::now();
    let mut timing = SplashTiming::default();
    assert_eq!(timing.visible_duration(start), None);
    timing.visible = Some(start);
    assert_eq!(
        timing.visible_duration(start + Duration::from_secs(2)),
        Some(Duration::from_secs(2))
    );
    timing.closed = Some(start + Duration::from_secs(3));
    assert_eq!(
        timing.visible_duration(start + Duration::from_secs(30)),
        Some(Duration::from_secs(3))
    );
}
