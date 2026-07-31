use serde::{Deserialize, Serialize};

macro_rules! define_job_enum {
    (
        $(#[$enum_meta:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            pub const ALL: [Self; [$(stringify!($variant)),+].len()] = [$(Self::$variant),+];
        }
    };
}

define_job_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub enum JobCategory {
        Import,
        Compile,
        Thumbnail,
        Export,
        Index,
        Play,
        Misc,
    }
}

define_job_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub enum JobPriority {
        Interactive,
        #[default]
        Normal,
        Background,
    }
}

impl JobPriority {
    pub(super) const fn admission_rank(self) -> u8 {
        match self {
            Self::Interactive => 0,
            Self::Normal => 1,
            Self::Background => 2,
        }
    }
}
