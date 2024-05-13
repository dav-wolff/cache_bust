use std::{path::PathBuf, str::FromStr};

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
	let mut local_path = PathBuf::from_str(literal.value()).expect("Expected a valid path");
	
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("assets");
	path.push(&local_path);
	
	let hashed_file_name = hashed_file_name(&path).expect("Error parsing file");
	
	if local_path.pop() {
		local_path.push(hashed_file_name);
	} else {
		local_path = hashed_file_name.into();
	}
	
	TokenTree::Literal(Literal::string(local_path.to_str().expect("Could not convert path to a string literal"))).into()
}
