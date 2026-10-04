mod host_interface;
mod registration;

#[cfg(feature = "backend-zr-vm")]
mod real_backend;
#[cfg(feature = "backend-zr-vm")]
mod state_encoding;
#[cfg(feature = "backend-zr-vm")]
mod support;
#[cfg(feature = "backend-zr-vm")]
mod vampire_product;
#[cfg(feature = "backend-zr-vm")]
mod woc_codec;
#[cfg(feature = "backend-zr-vm")]
mod woc_lifecycle;
