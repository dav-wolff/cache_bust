use std::{env, path::PathBuf, str::FromStr};

use cache_bust_core::hashed_file_name;
use litrs::StringLit;
use proc_macro::{Literal, TokenStream, TokenTree};

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
