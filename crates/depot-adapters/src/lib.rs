#[cfg(feature = "pypi")]
pub mod pypi;

#[cfg(feature = "npm")]
pub mod npm;

#[cfg(feature = "cargo-registry")]
pub mod cargo;

#[cfg(feature = "hex")]
pub mod hex;
