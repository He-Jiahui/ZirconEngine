//! #212: public linked Runtime witness. CPU admission/ingress and GPU pixels are separate gates.

#[path = "hud/owner.rs"]
mod owner;
#[path = "hud/pixels.rs"]
mod pixels;
#[path = "hud/project.rs"]
mod project;
#[path = "hud/wire.rs"]
mod wire;

use owner::{Menu, Sample, Tap};
use project::{Hud, Project};
use wire::{A11y, Session};

const CPU_BUTTON_POINT: [f32; 2] = [320.0, 244.0];
const OFF_BUTTON_POINT: [f32; 2] = [40.0, 300.0];

struct Arm {
    session: Session,
    tap: Tap,
    hud: Hud,
    menu: Menu,
}

impl Arm {
    fn open(profile: &'static str, hud: Hud, menu: Menu) -> Self {
        let project = Project::new(profile, hud);
        let tap = Tap::default();
        let session = Session::open(profile, project, tap.registration(menu));
        session.resize();
        Self {
            session,
            tap,
            hud,
            menu,
        }
    }

    fn tick(&self) -> Sample {
        self.session.tick();
        let sample = self.tap.take();
        sample.project.check(self.session.project_root(), self.hud);
        assert_eq!(
            sample.camera,
            project::CAMERA,
            "owner did not observe the authored scene camera"
        );
        assert!(sample.epoch != 0 && sample.world != 0 && sample.carrier_present);
        assert!(
            sample.journal.is_complete(),
            "owner did not obtain the real manager recording"
        );
        assert!(
            sample.ingress.generation.is_some(),
            "no real InputDriver attachment"
        );
        sample
    }

    fn seed(&self) -> (Sample, A11y) {
        self.tap.arm_seed();
        let sample = self.tick();
        sample.check_seed(self.menu);
        let a11y = self.session.a11y(self.hud);
        eprintln!(
            "hud-public {} admitted hud={:?} menu={:?} entity={} a11y={a11y:?} owner={sample:?}",
            self.session.tag(),
            self.hud,
            self.menu,
            project::CARRIER
        );
        (sample, a11y)
    }

    fn click(&self, before: &Sample, point: [f32; 2]) -> Sample {
        let codes = self.session.primary_click(point);
        let after = self.tick();
        eprintln!(
            "hud-public {} wire point={point:?} status={codes:?} before={before:?} after={after:?}",
            self.session.tag()
        );
        after.check_click(before, point);
        after
    }
}

#[test]
#[ignore = "requires an isolated linked-session test root; run with --ignored"]
fn public_hud_menu_cpu_no_root_slots() {
    for menu in [Menu::Start, Menu::Retry] {
        let arm = Arm::open("minimal", Hud::None, menu);
        let (initial, preview) = arm.seed();
        assert!(preview.caption.is_none());
        let off = arm.click(&initial, OFF_BUTTON_POINT);
        off.expect_unchanged(&initial, "CPU no-root off-button control");
        let on = arm.click(&off, CPU_BUTTON_POINT);
        on.expect_command(&off, menu, "CPU no-root in-button control (logic only)");
    }
}

fn cpu_hidden(menu: Menu) {
    let arm = Arm::open("minimal", Hud::Text, menu);
    let (initial, admitted) = arm.seed();
    let off = arm.click(&initial, OFF_BUTTON_POINT);
    off.expect_unchanged(&initial, "CPU authored off-button control");
    let after = arm.click(&off, CPU_BUTTON_POINT);
    assert_eq!(
        arm.session.a11y(Hud::Text),
        admitted,
        "authored a11y identity/focus/bounds changed"
    );
    eprintln!("hud-public cpu behavior-ready hud=Text menu={menu:?}");
    after.expect_unchanged(
        &off,
        "CPU admitted HUD: the fallback must not write the control slot",
    );
}

#[test]
#[ignore = "requires an isolated linked-session test root; run with --ignored"]
fn public_hud_menu_cpu_declared_hud_keeps_start_inert() {
    cpu_hidden(Menu::Start);
}

#[test]
#[ignore = "requires an isolated linked-session test root; run with --ignored"]
fn public_hud_menu_cpu_declared_hud_keeps_retry_inert() {
    cpu_hidden(Menu::Retry);
}

#[test]
#[ignore = "requires real headless WGPU adapter, product pipelines, font and RGBA readback"]
fn public_hud_menu_gpu_framebuffer_pairs_visible_and_hidden_start() {
    let (clean, point, button) = {
        let control = Arm::open("runtime", Hud::None, Menu::Start);
        let no_menu = control.tick();
        no_menu.check_unseeded();
        let clean = pixels::capture_until(&control.session, "no-root/no-menu background", |_| true);
        let (before, _) = control.seed();
        before.check_successor(&no_menu);
        let visible = pixels::capture_until(&control.session, "no-root/Start accent", |frame| {
            frame.button().is_some()
        });
        assert!(
            visible.generation > clean.generation,
            "stale no-root capture"
        );
        let button = visible.button().expect("GPU gate: rendered Start button");
        let point = pixels::check_visible_control(&clean, &visible, button);
        let after = control.click(&before, point);
        after.expect_command(&before, Menu::Start, "GPU visible Start control");
        (clean, point, button)
    };

    let rooted = Arm::open("runtime", Hud::TextAndImage, Menu::Start);
    let (before, admitted) = rooted.seed();
    // The first rooted submission is AFTER the menu is seeded; no pre-seed rooted frame can
    // enter the asynchronous readback mailbox. The matched clean scene is the no-root baseline.
    let frame = pixels::capture_until(&rooted.session, "authored Image and Text", |frame| {
        frame.hud().visible()
    });
    pixels::check_rooted(&clean, &frame, button);
    let after = rooted.click(&before, point);
    assert_eq!(
        rooted.session.a11y(Hud::TextAndImage),
        admitted,
        "authored a11y identity/focus/bounds changed"
    );
    eprintln!(
        "hud-public gpu behavior-ready generation={} old-button={button:?}",
        frame.generation
    );
    after.expect_unchanged(
        &before,
        "GPU real HUD and absent Start: the fallback must not write the control slot",
    );
}
