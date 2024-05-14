#[cfg(feature = "macro")]
pub use cache_bust_macro::asset;

#[cfg(feature = "build")]
mod cache_bust;
#[cfg(feature = "build")]
pub use cache_bust::*;
