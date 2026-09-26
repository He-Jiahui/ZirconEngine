//! 编译期模板文件清单，同时决定发布程序的渲染内容与模板内容摘要。
//! 增删模板文件须同步这里；运行时不会回读源码树来发现遗漏的文件。

pub(super) struct EmbeddedProjectTemplateEntry {
    pub path: &'static str,
    pub bytes: &'static [u8],
}

macro_rules! template_bytes {
    ($path:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../templates/projects/renderable-empty/",
            $path
        ))
    };
}

pub(super) const RENDERABLE_EMPTY_ENTRIES: &[EmbeddedProjectTemplateEntry] = &[
    EmbeddedProjectTemplateEntry {
        path: ".gitignore",
        bytes: template_bytes!(".gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/autosave/.gitignore",
        bytes: template_bytes!(".zircon/autosave/.gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/cache/.gitignore",
        bytes: template_bytes!(".zircon/cache/.gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/play/.gitignore",
        bytes: template_bytes!(".zircon/play/.gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/registry/.gitignore",
        bytes: template_bytes!(".zircon/registry/.gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/settings.toml",
        bytes: template_bytes!(".zircon/settings.toml"),
    },
    EmbeddedProjectTemplateEntry {
        path: ".zircon/thumbnails/.gitignore",
        bytes: template_bytes!(".zircon/thumbnails/.gitignore"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/materials/default.zmaterial",
        bytes: template_bytes!("assets/materials/default.zmaterial"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/materials/default.zmaterial.zmeta",
        bytes: template_bytes!("assets/materials/default.zmaterial.zmeta"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/models/cube.obj",
        bytes: template_bytes!("assets/models/cube.obj"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/models/cube.obj.zmeta",
        bytes: template_bytes!("assets/models/cube.obj.zmeta"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/scenes/main.scene.toml",
        bytes: template_bytes!("assets/scenes/main.scene.toml"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/shaders/pbr_shader.zmeta",
        bytes: template_bytes!("assets/shaders/pbr_shader.zmeta"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/shaders/pbr_shader/pbr.wgsl",
        bytes: template_bytes!("assets/shaders/pbr_shader/pbr.wgsl"),
    },
    EmbeddedProjectTemplateEntry {
        path: "assets/shaders/pbr_shader/pbr.zshader",
        bytes: template_bytes!("assets/shaders/pbr_shader/pbr.zshader"),
    },
    EmbeddedProjectTemplateEntry {
        path: "export/desktop_windows.zpreset",
        bytes: template_bytes!("export/desktop_windows.zpreset"),
    },
    EmbeddedProjectTemplateEntry {
        path: "zircon-project.toml",
        bytes: template_bytes!("zircon-project.toml"),
    },
];
