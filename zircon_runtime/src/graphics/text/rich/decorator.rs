use crate::core::framework::render::StyleOverride;

use super::bbcode::apply_builtin_style;

pub(super) trait TextDecorator: Send + Sync {
    fn tag(&self) -> &str;
    fn apply(&self, value: Option<&str>, style: &mut StyleOverride);
}

struct BuiltinTextDecorator {
    tag: &'static str,
}

impl TextDecorator for BuiltinTextDecorator {
    fn tag(&self) -> &str {
        self.tag
    }

    fn apply(&self, value: Option<&str>, style: &mut StyleOverride) {
        let _ = apply_builtin_style(self.tag, value, style);
    }
}

pub(super) struct DecoratorRegistry {
    decorators: Vec<Box<dyn TextDecorator>>,
}

impl DecoratorRegistry {
    pub(super) fn with_builtins() -> Self {
        let decorators = [
            "b", "i", "u", "s", "color", "bgcolor", "size", "font", "code",
        ]
        .into_iter()
        .map(|tag| Box::new(BuiltinTextDecorator { tag }) as Box<dyn TextDecorator>)
        .collect();
        Self { decorators }
    }

    pub(super) fn apply(&self, tag: &str, value: Option<&str>, style: &mut StyleOverride) -> bool {
        let Some(decorator) = self
            .decorators
            .iter()
            .find(|decorator| decorator.tag() == tag)
        else {
            return false;
        };
        decorator.apply(value, style);
        true
    }
}
