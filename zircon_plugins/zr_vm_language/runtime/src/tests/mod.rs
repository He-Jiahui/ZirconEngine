#[path = "host_interface.rs"]
mod host_interface;
#[path = "registration.rs"]
mod registration;

#[cfg(feature = "backend-zr-vm")]
#[path = "real_backend.rs"]
mod real_backend;
#[cfg(feature = "backend-zr-vm")]
#[path = "state_encoding.rs"]
mod state_encoding;
#[cfg(feature = "backend-zr-vm")]
#[path = "support.rs"]
mod support;
#[cfg(feature = "backend-zr-vm")]
#[path = "vampire_product.rs"]
mod vampire_product;
#[cfg(feature = "backend-zr-vm")]
#[path = "woc_codec.rs"]
mod woc_codec;
#[cfg(feature = "backend-zr-vm")]
#[path = "woc_lifecycle.rs"]
mod woc_lifecycle;
