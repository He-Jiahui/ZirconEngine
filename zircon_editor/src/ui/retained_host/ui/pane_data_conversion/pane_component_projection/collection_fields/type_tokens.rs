#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CollectionTypeTraits {
    generic: bool,
    numeric: bool,
    asset: bool,
    instance: bool,
    object: bool,
    reference: bool,
    string: bool,
    boolean: bool,
    color: bool,
    vector: bool,
    vec2: bool,
    vec3: bool,
    vec4: bool,
}

impl CollectionTypeTraits {
    pub(super) fn from_declared_type(declared_type: &str) -> Self {
        let contains = |token| contains_ignore_ascii_case(declared_type, token);
        Self {
            generic: declared_type.is_empty()
                || ["any", "value", "uivalue", "variant", "unknown"]
                    .iter()
                    .any(|token| declared_type.eq_ignore_ascii_case(token)),
            numeric: contains("int")
                || contains("float")
                || contains("double")
                || contains("number"),
            asset: contains("asset"),
            instance: contains("instance"),
            object: contains("object"),
            reference: contains("ref"),
            string: contains("string"),
            boolean: contains("bool"),
            color: contains("color"),
            vector: contains("vec") || contains("vector"),
            vec2: contains("vec2"),
            vec3: contains("vec3"),
            vec4: contains("vec4"),
        }
    }

    pub(super) const fn is_generic(self) -> bool {
        self.generic
    }

    pub(super) const fn is_numeric(self) -> bool {
        self.numeric
    }

    pub(super) const fn is_reference_like(self) -> bool {
        self.asset || self.instance || self.object || self.reference
    }

    pub(super) const fn is_asset(self) -> bool {
        self.asset
    }

    pub(super) const fn is_object_like(self) -> bool {
        self.instance || self.object
    }

    pub(super) const fn is_reference(self) -> bool {
        self.reference
    }

    pub(super) const fn is_string(self) -> bool {
        self.string
    }

    pub(super) const fn is_boolean(self) -> bool {
        self.boolean
    }

    pub(super) const fn is_color(self) -> bool {
        self.color
    }

    pub(super) const fn is_vector(self) -> bool {
        self.vector
    }

    pub(super) const fn is_vec2(self) -> bool {
        self.vec2
    }

    pub(super) const fn is_vec3(self) -> bool {
        self.vec3
    }

    pub(super) const fn is_vec4(self) -> bool {
        self.vec4
    }
}

fn contains_ignore_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
