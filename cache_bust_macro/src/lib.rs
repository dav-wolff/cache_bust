#![forbid(unsafe_code)]
#![deny(non_snake_case)]
#![warn(missing_docs)]

//! Procedural macro for **[cache_bust]**
//! 
//! [cache_bust]: https://github.com/dav-wolff/cache_bust

use std::{env, path::PathBuf, str::FromStr};

use cache_bust_core::hashed_file_name;
use litrs::StringLit;
use proc_macro::{Literal, TokenStream, TokenTree};

/// Converts a file location to its hashed equivalent (e.g. `images/circle.png`
/// to `images/circle-f04a[...].png`).
/// 
/// By default this will look for assets in the `assets` directory inside your crate.
/// To use a different directory set the `CACHE_BUST_ASSETS_DIR` environment variable.
/// If the file doesn't exist, the macro will produce an error.
/// 
/// It's also possible to use an absolute path like `/images/circle.png`. This path will
/// still be looked up relative to the assets directory and results in
/// `images/circle-f04a[...].png`.
/// 
/// The hashing of the file name can also be disabled by setting the `CACHE_BUST_SKIP_HASHING`
/// environment variable to `1`. In this case the macro will act as an identity function,
/// while still erroring if the file doesn't exist. This can be useful if hashing is
/// only wanted in some builds but not others.
/// 
/// # Examples
/// 
/// ```
/// # use cache_bust_macro as cache_bust;
/// use cache_bust::asset;
/// 
/// assert_eq!(asset!("images/circle.png"), "images/circle-f04a632bf7de8a58d730988671a9139d6f7b3b197bbc78b6c74a4542eaa4878d.png");
/// assert_eq!(asset!("/images/circle.png"), "/images/circle-f04a632bf7de8a58d730988671a9139d6f7b3b197bbc78b6c74a4542eaa4878d.png");
/// ```
/// 
/// Compiled with `CACHE_BUST_SKIP_HASHING=1`:
/// ```rust,ignore
/// assert_eq!(asset!("images/circle.png"), "images/circle.png");
/// assert_eq!(asset!("/images/circle.png"), "/images/circle.png");
/// ```
#[proc_macro]
pub fn asset(token_stream: TokenStream) -> TokenStream {
	let mut iter = token_stream.into_iter();
	let token = iter.next().expect("Expected file name as a string");
	
	if iter.next().is_some() {
		panic!("Expected file name as a string");
	}
	
	let literal = StringLit::try_from(token).expect("Expected file name as a string");
	
	let (local_path, is_absolute) = if literal.value().starts_with('/') {
		(&literal.value()[1..], true)
	} else {
		(literal.value(), false)
	};
	
	let assets_dir = env::var_os("CACHE_BUST_ASSETS_DIR").unwrap_or("assets".into());
	
	let mut local_path = PathBuf::from_str(local_path).expect("Expected a valid path");
	let mut path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should exist"));
	path.push(assets_dir);
	path.push(&local_path);
	
	let mut hashed_file_name = hashed_file_name(&path).unwrap_or_else(|err| panic!("Error parsing file {path:?}: {err}"));
	
	// only revert the file_name after hashing to keep the same error reporting
	if env::var("CACHE_BUST_SKIP_HASHING").is_ok_and(|skip_hashing| skip_hashing == "1") {
		path.file_name().expect("File name is existent").clone_into(&mut hashed_file_name);
	}
	
	if local_path.pop() {
		local_path.push(hashed_file_name);
	} else {
		local_path = hashed_file_name.into();
	}
	
	let local_path = local_path.to_str().expect("Could not convert path to a string literal");
	
	let literal = if is_absolute {
		Literal::string(&format!("/{local_path}"))
	} else {
		Literal::string(local_path)
	};
	
	TokenTree::Literal(literal).into()
}
