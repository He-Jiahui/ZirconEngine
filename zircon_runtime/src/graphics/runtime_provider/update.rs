#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimeProviderUpdate<S> {
    stats: S,
}

impl<S> RuntimeProviderUpdate<S> {
    pub(crate) fn new(stats: S) -> Self {
        Self { stats }
    }

    pub(crate) fn stats(&self) -> &S {
        &self.stats
    }
}

impl<S: Copy> RuntimeProviderUpdate<S> {
    pub(crate) fn stats_copy(&self) -> S {
        self.stats
    }
}

macro_rules! define_runtime_provider_update {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            stats: $stats_ty:ty => copy;
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        $vis struct $name {
            update: RuntimeProviderUpdate<$stats_ty>,
        }

        impl $name {
            pub fn new(stats: $stats_ty) -> Self {
                Self {
                    update: RuntimeProviderUpdate::new(stats),
                }
            }

            pub fn stats(&self) -> $stats_ty {
                self.update.stats_copy()
            }
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            stats: $stats_ty:ty => ref;
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        $vis struct $name {
            update: RuntimeProviderUpdate<$stats_ty>,
        }

        impl $name {
            pub fn new(stats: $stats_ty) -> Self {
                Self {
                    update: RuntimeProviderUpdate::new(stats),
                }
            }

            pub fn stats(&self) -> &$stats_ty {
                self.update.stats()
            }
        }
    };
}

pub(crate) use define_runtime_provider_update;
