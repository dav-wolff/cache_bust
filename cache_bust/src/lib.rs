#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

//! A library for compile-time "cache busting", including hashes in file names in order to optimize for caching.
//! 
//! # Why cache busting?
//! 
//! Cache busting is primarily applicable in web applications, optimizing the usage of the browser's cache.
//! By including a hash of the contents of a file in its name, it's made possible to [declare said file
//! as immutable in the `Cache-Control` HTTP-Header][`Cache-Control-immutable`] without losing the ability
//! to push updates to the file out to the browser. As long as the file isn't changed, it can remain cached.
//! As soon as it's changed, the browser will simply look it up under its new name.
//! 
//! # How to use cache_bust
//! 
//! ## The `asset!` macro
//! 
//! To include an asset in source code, use the [`asset!`][`asset`] macro:
//! ```
//! use cache_bust::asset;
//! 
//! let img_src = asset!("images/circle.png");
//! assert_eq!(img_src, "images/circle-f04a632bf7de8a58d730988671a9139d6f7b3b197bbc78b6c74a4542eaa4878d.png");
//! ```
//! 
//! By default this will look for assets in the `assets` directory inside your crate.
//! To use a different directory set the `CACHE_BUST_ASSETS_DIR` environment variable.
//! If the file doesn't exist, the macro will produce an error.
//! 
//! The hashing of the file name can also be disabled, for example for debug builds
//! where cache busting isn't being used, by setting the `CACHE_BUST_SKIP_HASHING`
//! environment variable to `1`. In this case the macro will act as an identity function,
//! while still erroring if the file doesn't exist.
//! 
//! ## Build time
//! 
//! The next step is to rename the files on disk to include their hashes.
//! This can occur either in-place, or the renamed files can be copied to
//! a new location, depending on where your runtime expects the assets.
//! 
//! There are two ways to achieve this: using a Rust API, or using a CLI-tool.
//! 
//! ### Rust API
//! 
//! The Rust API can be called from a `build.rs` build script:
//! ```
//! # fn main() -> Result<(), std::io::Error> {
//! use cache_bust::CacheBust;
//! 
//! let cache_bust = CacheBust::builder()
//! 	.out_dir("hashed_assets".to_owned())
//! 	.build();
//! 
//! cache_bust.hash_dir()?;
//! assert_eq!(
//! 	std::fs::read("assets/images/circle.png")?,
//! 	std::fs::read("hashed_assets/images/circle-f04a632bf7de8a58d730988671a9139d6f7b3b197bbc78b6c74a4542eaa4878d.png")?
//! );
//! # Ok(())
//! # }
//! ```
//! 
//! ### CLI-tool
//! 
//! Alternatively **cache_bust_cli** can be used from some other build tool
//! to hash and rename the files:
//! ```sh
//! cachebust assets --out hashed_assets
//! ```
//! 
//! ## Dynamic files
//! 
//! Some files might be dynamically generated during build time and thus not
//! possible to include using the [`asset!`][`asset`] macro. It's possible to
//! individually hash these files and obtain their hashed names at build time.
//! How to pass those names to the runtime is left to you.
//! 
//! ### Rust API
//! 
//! ```
//! # fn main() -> Result<(), std::io::Error> {
//! use cache_bust::CacheBust;
//! 
//! let cache_bust = CacheBust::builder()
//! 	.out_dir("hashed_assets".to_owned())
//! 	.build();
//! 
//! let path = cache_bust.hash_file("generated/script.js")?;
//! assert_eq!(
//! 	std::fs::read_to_string(path)?,
//! 	"alert('Hello world');\n"
//! );
//! # Ok(())
//! # }
//! ```
//! 
//! ### CLI-tool
//! 
//! ```sh
//! cachebust assets --file generated/script.js --out hashed_assets --print-file-name
//! cachebust assets --file generated/script.js --out hashed_assets --print-file-path
//! ```
//! 
//! # Features
//! 
//! ### default
//! 
//! Enables all features.
//! 
//! ### macro
//! 
//! Enables the `asset!` procedural macro.
//! 
//! ### build
//! 
//! Enables the `CacheBust` and `CacheBustBuilder` structs for hashing files at build time.
//! 
//! [`Cache-Control-immutable`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control#caching_static_assets_with_cache_busting

#[cfg(feature = "macro")]
#[doc(inline)]
pub use cache_bust_macro::asset;

#[cfg(feature = "build")]
mod cache_bust;
#[cfg(feature = "build")]
pub use cache_bust::*;
