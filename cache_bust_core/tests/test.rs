use std::path::PathBuf;

use cache_bust_core::*;

#[test]
fn test_hello_txt() {
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests");
	path.push("hello.txt");
	let hashed_name = hashed_file_name(&path).unwrap();
	
	assert_eq!(hashed_name, "hello-d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5.txt");
}

#[test]
fn test_hello() {
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests");
	path.push("hello");
	let hashed_name = hashed_file_name(&path).unwrap();
	
	assert_eq!(hashed_name, "hello-97f24948156c5ea491bda3d05d12b334c57409e3b746e73215585b2fe99fb098");
}
