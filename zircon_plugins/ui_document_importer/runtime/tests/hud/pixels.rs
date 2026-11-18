use std::{
    thread,
    time::{Duration, Instant},
};

use super::wire::{Frame, Session, HEIGHT, WIDTH};

const ALL: Rect = Rect {
    lo: [0, 0],
    hi: [WIDTH, HEIGHT],
};
const MENU_SEARCH: Rect = Rect {
    lo: [100, 190],
    hi: [540, 300],
};
const CYAN_PATCH: Rect = Rect {
    lo: [32, 30],
    hi: [47, 50],
};
const GOLD_PATCH: Rect = Rect {
    lo: [65, 30],
    hi: [80, 50],
};
const CAPTION: Rect = Rect {
    lo: [120, 24],
    hi: [370, 64],
};

#[derive(Clone, Copy, Debug)]
pub(super) struct Rect {
    lo: [u32; 2],
    hi: [u32; 2],
} // half-open pixel bounds

impl Rect {
    fn points(self) -> impl Iterator<Item = [u32; 2]> {
        (self.lo[1]..self.hi[1]).flat_map(move |y| (self.lo[0]..self.hi[0]).map(move |x| [x, y]))
    }
    fn area(self) -> usize {
        ((self.hi[0] - self.lo[0]) * (self.hi[1] - self.lo[1])) as usize
    }
}

impl Frame {
    fn pixel(&self, [x, y]: [u32; 2]) -> [u8; 4] {
        let offset = ((y * WIDTH + x) * 4) as usize;
        self.rgba[offset..offset + 4]
            .try_into()
            .expect("RGBA pixel")
    }

    fn count(&self, rectangle: Rect, matches: impl Fn([u8; 4]) -> bool) -> usize {
        rectangle
            .points()
            .filter(|&p| matches(self.pixel(p)))
            .count()
    }

    fn changed(&self, other: &Self, rectangle: Rect) -> usize {
        rectangle
            .points()
            .filter(|&p| {
                self.pixel(p)
                    .into_iter()
                    .zip(other.pixel(p))
                    .any(|(a, b)| a.abs_diff(b) > 8)
            })
            .count()
    }

    fn real(&self) -> bool {
        self.generation != 0
            && self
                .rgba
                .chunks_exact(4)
                .any(|p| p[3] >= 240 && p[..3].iter().any(|&v| v >= 20))
    }

    pub fn button(&self) -> Option<Rect> {
        let mut lo = MENU_SEARCH.hi;
        let mut hi = MENU_SEARCH.lo;
        let mut count = 0;
        for point in MENU_SEARCH
            .points()
            .filter(|&p| is_start_accent(self.pixel(p)))
        {
            count += 1;
            for i in 0..2 {
                lo[i] = lo[i].min(point[i]);
                hi[i] = hi[i].max(point[i] + 1);
            }
        }
        // Rounded corners, the thin pale border, and the label may cut into the red fill.
        (count >= 7_000
            && (240..=290).contains(&hi[0].saturating_sub(lo[0]))
            && (36..=52).contains(&hi[1].saturating_sub(lo[1])))
        .then_some(Rect { lo, hi })
    }

    pub fn hud(&self) -> HudPixels {
        HudPixels {
            cyan: self.count(CYAN_PATCH, |[r, g, b, a]| {
                r <= 105 && g >= 180 && b >= 200 && a >= 240
            }),
            gold: self.count(GOLD_PATCH, |[r, g, b, a]| {
                r >= 195 && g >= 125 && r > g && b <= 110 && a >= 240
            }),
            green: self.count(CAPTION, |[r, g, b, a]| {
                r < 105 && g >= 175 && b <= 145 && g > b.saturating_add(35) && a >= 240
            }),
        }
    }
}

#[derive(Debug)]
pub(super) struct HudPixels {
    cyan: usize,
    gold: usize,
    green: usize,
}

impl HudPixels {
    pub fn visible(&self) -> bool {
        self.cyan >= CYAN_PATCH.area() * 9 / 10
            && self.gold >= GOLD_PATCH.area() * 9 / 10
            && (100..CAPTION.area() / 2).contains(&self.green)
    }
}

fn is_start_accent([r, g, b, a]: [u8; 4]) -> bool {
    let near = |expected: [u8; 3]| {
        [r, g, b]
            .into_iter()
            .zip(expected)
            .all(|(actual, nominal)| actual.abs_diff(nominal) <= 15)
    };
    // Authored fill #d43a4b and its linear-to-sRGB encoding, depending on terminal transfer.
    a >= 240 && (near([212, 58, 75]) || near([235, 131, 148]))
}

pub(super) fn capture_until(
    session: &Session,
    stage: &str,
    accept: impl Fn(&Frame) -> bool,
) -> Frame {
    let mut last_seen = 0;
    let mut last_real = None;
    let mut deadline = None;
    for _ in 0..48 {
        let frame = session.frame();
        let until = *deadline.get_or_insert_with(|| Instant::now() + Duration::from_secs(45));
        if frame.generation != 0 {
            assert!(
                frame.generation > last_seen,
                "GPU gate {stage}: stale generation {} after {last_seen}",
                frame.generation
            );
            last_seen = frame.generation;
        }
        if frame.real() {
            if accept(&frame) {
                return frame;
            }
            last_real = Some((
                frame.generation,
                frame.button(),
                frame.hud(),
                frame.count(ALL, is_start_accent),
            ));
        }
        if Instant::now() >= until {
            break;
        }
        thread::sleep(Duration::from_millis(12));
    }
    panic!("GPU precondition {stage}: no validated framebuffer ROI; last_generation={last_seen} last_real={last_real:?}");
}

pub(super) fn check_visible_control(clean: &Frame, visible: &Frame, button: Rect) -> [f32; 2] {
    let red = visible.count(button, is_start_accent);
    let outer = visible.count(ALL, is_start_accent) - red;
    let clean_red = clean.count(ALL, is_start_accent);
    let changed = visible.changed(clean, button);
    assert!(clean_red == 0 && outer < 10 && changed >= red, "GPU visible control invalid: clean_red={clean_red}, outer={outer}, red={red}, changed={changed}");
    let center_x = (button.lo[0] + button.hi[0]) / 2;
    let y = (button.lo[1] + 6..button.hi[1] - 6)
        .find(|&y| {
            let square = Rect {
                lo: [center_x - 4, y - 4],
                hi: [center_x + 5, y + 5],
            };
            visible.count(square, is_start_accent) == square.area()
        })
        .expect("GPU visible control lacks a real 9 x 9 accent patch near button center");
    let point = [center_x as f32 + 0.5, y as f32 + 0.5];
    eprintln!("hud-public gpu visible pixels baseline={} frame={} rect={button:?} red={red} exterior={outer} changed={changed} point={point:?}", clean.generation, visible.generation);
    point
}

pub(super) fn check_rooted(clean: &Frame, frame: &Frame, button: Rect) {
    let clean_hud = clean.hud();
    let hud = frame.hud();
    let red = frame.count(ALL, is_start_accent);
    let changed = clean.changed(frame, button);
    eprintln!("hud-public gpu rooted pixels generation={} no-root baseline={} baseline_hud={clean_hud:?} hud={hud:?} rect={button:?} old-accent={red} changed={changed}/{}", frame.generation, clean.generation, button.area());
    assert!(
        clean_hud.cyan < 5 && clean_hud.gold < 5 && clean_hud.green < 5 && hud.visible(),
        "GPU HUD positive controls: two distinct texture patches and bounded green glyphs"
    );
    assert_eq!(
        red, 0,
        "GPU visibility: old Start accent is still visible beside the authored HUD"
    );
    assert!(
        changed <= button.area() / 100,
        "GPU visibility: background under old button differs from the matched clean scene"
    );
}
