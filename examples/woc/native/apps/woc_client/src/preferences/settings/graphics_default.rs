const AMPLE_DEVICE_MEMORY_GIB: f64 = 8.0;
const AMPLE_LOGICAL_CORES: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuClass {
    Software,
    StrongDesktop,
    FlagshipMobile,
    MidIntegrated,
    MidMobile,
    Weak,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum GraphicsPreset {
    Low = 1,
    Medium = 2,
    High = 3,
    Ultra = 4,
    Advanced = 5,
}

impl GraphicsPreset {
    pub const fn setting_value(self) -> f64 {
        self as u8 as f64
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GraphicsRuntimeHints<'a> {
    pub device_memory_gib: Option<f64>,
    pub hardware_concurrency: Option<u32>,
    pub max_touch_points: u32,
    pub coarse_pointer: bool,
    pub narrow_viewport: bool,
    pub gpu_renderer: Option<&'a str>,
}

impl GraphicsRuntimeHints<'_> {
    pub fn is_touch_device(&self) -> bool {
        self.max_touch_points > 0 && (self.coarse_pointer || self.narrow_viewport)
    }
}

pub fn classify_gpu_renderer(renderer: Option<&str>) -> GpuClass {
    let name = renderer.unwrap_or_default().to_ascii_lowercase();
    if name.is_empty() {
        return GpuClass::Unknown;
    }
    if contains_any(
        &name,
        &[
            "swiftshader",
            "llvmpipe",
            "basic render",
            "softpipe",
            "microsoft basic",
            "software",
        ],
    ) {
        return GpuClass::Software;
    }
    if is_weak_integrated_gpu(&name) {
        return GpuClass::Weak;
    }
    if contains_word(&name, "rtx")
        || contains_word(&name, "gtx")
        || name.contains("geforce")
        || is_strong_radeon(&name)
        || contains_word(&name, "arc")
        || contains_word(&name, "nvidia")
        || number_after(&name, "apple m").is_some_and(|generation| (1..=9).contains(&generation))
    {
        return GpuClass::StrongDesktop;
    }

    let apple_a = number_after(&name, "apple a");
    let adreno = number_after(&name, "adreno (tm) ");
    let mali_g = number_after(&name, "mali-g");
    if apple_a.is_some_and(|generation| (14..=99).contains(&generation))
        || adreno.is_some_and(|generation| (660..=899).contains(&generation))
        || name.contains("immortalis")
        || mali_g.is_some_and(|generation| (700..=799).contains(&generation))
        || name.contains("xclipse")
    {
        return GpuClass::FlagshipMobile;
    }

    let radeon_family = radeon_family(&name);
    let graphics_generation = graphics_generation(&name);
    if name.contains("iris xe")
        || name.contains("iris plus")
        || matches!(radeon_family, Some("vega" | "graphics"))
        || graphics_generation.is_some_and(|(family, generation)| {
            family == "uhd" && (700..=799).contains(&generation)
        })
        || (name.contains("intel") && contains_word(&name, "xe"))
    {
        return GpuClass::MidIntegrated;
    }

    let power_vr_family = token_after(&name, "powervr ");
    if apple_a.is_some_and(|generation| (11..=13).contains(&generation))
        || adreno.is_some_and(|generation| (500..=659).contains(&generation))
        || mali_g.is_some_and(|generation| {
            (53..=59).contains(&generation)
                || (60..=69).contains(&generation)
                || (70..=78).contains(&generation)
        })
        || power_vr_family.is_some_and(|family| {
            family.starts_with("gt") || family.starts_with("gm") || family.starts_with('b')
        })
    {
        return GpuClass::MidMobile;
    }

    let mali = number_after(&name, "mali-");
    if adreno.is_some_and(|generation| (300..=499).contains(&generation))
        || name.contains("mali-t")
        || mali.is_some_and(|generation| (400..=499).contains(&generation))
        || matches!(mali_g, Some(31 | 51 | 52))
        || power_vr_family
            .is_some_and(|family| family.starts_with("sgx") || family.starts_with("g6"))
        || apple_a.is_some_and(|generation| (5..=10).contains(&generation))
        || graphics_generation.is_some_and(|(_, generation)| (100..=999).contains(&generation))
        || (name.contains("intel") && contains_word(&name, "gma"))
    {
        return GpuClass::Weak;
    }
    GpuClass::Unknown
}

pub fn resolve_default_graphics_preset(hints: &GraphicsRuntimeHints<'_>) -> GraphicsPreset {
    let gpu = classify_gpu_renderer(hints.gpu_renderer);
    let ample_or_unknown_memory = hints.device_memory_gib.is_none()
        || hints
            .device_memory_gib
            .is_some_and(|memory| memory >= AMPLE_DEVICE_MEMORY_GIB)
        || hints
            .hardware_concurrency
            .is_some_and(|cores| cores >= AMPLE_LOGICAL_CORES);

    if matches!(gpu, GpuClass::Software | GpuClass::Weak) {
        return GraphicsPreset::Low;
    }
    if gpu == GpuClass::StrongDesktop && !hints.is_touch_device() {
        return if ample_or_unknown_memory {
            GraphicsPreset::Ultra
        } else {
            GraphicsPreset::High
        };
    }
    if gpu == GpuClass::FlagshipMobile
        || (gpu == GpuClass::StrongDesktop && hints.is_touch_device())
    {
        return GraphicsPreset::High;
    }
    if matches!(gpu, GpuClass::MidIntegrated | GpuClass::MidMobile) {
        return GraphicsPreset::Medium;
    }
    if gpu == GpuClass::Unknown
        && !hints.is_touch_device()
        && hints
            .device_memory_gib
            .is_some_and(|memory| memory >= AMPLE_DEVICE_MEMORY_GIB)
        && hints
            .hardware_concurrency
            .is_some_and(|cores| cores >= AMPLE_LOGICAL_CORES)
    {
        return GraphicsPreset::High;
    }
    GraphicsPreset::Medium
}

pub fn first_run_graphics_preset(
    default_already_applied: bool,
    hints: &GraphicsRuntimeHints<'_>,
) -> Option<GraphicsPreset> {
    if default_already_applied {
        return None;
    }
    let preset = resolve_default_graphics_preset(hints);
    (preset != GraphicsPreset::Medium).then_some(preset)
}

pub fn native_safe_graphics_preset(preset: GraphicsPreset) -> GraphicsPreset {
    if preset >= GraphicsPreset::Ultra {
        GraphicsPreset::High
    } else {
        preset
    }
}

fn is_weak_integrated_gpu(name: &str) -> bool {
    name.contains("intel")
        && contains_any(
            name,
            &[
                "iris(tm) plus graphics 6",
                "iris plus graphics 6",
                "uhd graphics 6",
                "hd graphics 5",
                "hd graphics 6",
            ],
        )
}

fn is_strong_radeon(name: &str) -> bool {
    radeon_family(name).is_some_and(|family| {
        family.starts_with("rx") || family.starts_with("pro") || family.starts_with("vii")
    })
}

fn radeon_family(name: &str) -> Option<&str> {
    let tail = name.split_once("radeon")?.1.trim_start();
    let tail = tail.strip_prefix("(tm)").unwrap_or(tail).trim_start();
    token_after(tail, "")
}

fn graphics_generation(name: &str) -> Option<(&'static str, u32)> {
    for (family, marker) in [("uhd", "uhd graphics "), ("hd", "hd graphics ")] {
        if let Some(generation) = number_after(name, marker) {
            return Some((family, generation));
        }
    }
    None
}

fn number_after(name: &str, marker: &str) -> Option<u32> {
    let tail = name.split_once(marker)?.1;
    let digits = tail
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn token_after<'a>(name: &'a str, marker: &str) -> Option<&'a str> {
    let tail = if marker.is_empty() {
        name
    } else {
        name.split_once(marker)?.1
    };
    tail.split_whitespace().next()
}

fn contains_any(name: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| name.contains(pattern))
}

fn contains_word(name: &str, word: &str) -> bool {
    name.match_indices(word).any(|(index, _)| {
        let before = name[..index].chars().next_back();
        let after = name[index + word.len()..].chars().next();
        before.is_none_or(|character| !character.is_ascii_alphanumeric())
            && after.is_none_or(|character| !character.is_ascii_alphanumeric())
    })
}
