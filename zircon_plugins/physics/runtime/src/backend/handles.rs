macro_rules! backend_handle {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(u64);

        impl $name {
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl ArenaHandle for $name {
            fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

pub(super) trait ArenaHandle: Copy {
    fn from_raw(raw: u64) -> Self;
    fn raw(self) -> u64;
}

backend_handle!(BodyHandle);
backend_handle!(ShapeHandle);
backend_handle!(ConstraintHandle);
