use std::{env, fs::{self, File}, path::PathBuf};

use cache_bust::CacheBust;

fn assets_dir() -> PathBuf {
	let mut dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
	dir.push("assets");
	dir
}

fn create_temp_dir(test: &'static str) -> PathBuf {
	let mut dir: PathBuf = env!("CARGO_TARGET_TMPDIR").into();
	dir.push(test);
	let _ = fs::remove_dir_all(&dir);
	fs::create_dir(&dir).unwrap();
	dir
}

#[test]
fn out_dir_is_cleared() {
	let temp_dir = create_temp_dir("out_dir_is_cleared");
	
	let mut file_to_delete = temp_dir.clone();
	file_to_delete.push("file_to_delete");
	File::create(&file_to_delete).unwrap();
	
	CacheBust::builder()
		.out_dir(temp_dir)
		.build()
		.execute().unwrap();
	
	assert!(!file_to_delete.exists());
}

#[test]
fn in_to_out_dir() {
	let temp_dir = create_temp_dir("in_to_out_dir");
	
	CacheBust::builder()
		.out_dir(temp_dir.clone())
		.build()
		.execute().unwrap();
	
	let mut hello = assets_dir();
	hello.push("hello.txt");
	
	let mut hi = assets_dir();
	hi.push("greetings");
	hi.push("hi.txt");
	
	let mut hello_hashed = temp_dir.clone();
	hello_hashed.push("hello-d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5.txt");
	
	let mut hi_hashed = temp_dir;
	hi_hashed.push("greetings");
	hi_hashed.push("hi-c01a4cfa25cb895cdd0bb25181ba9c1622e93895a6de6f533a7299f70d6b0cfb.txt");
	
	assert_eq!(fs::read(hello).unwrap(), fs::read(hello_hashed).unwrap());
	assert_eq!(fs::read(hi).unwrap(), fs::read(hi_hashed).unwrap());
}

#[test]
fn in_place() {
	let temp_dir = create_temp_dir("in_place");
	
	let mut empty = temp_dir.clone();
	empty.push("empty");
	File::create(&empty).unwrap();
	
	let mut some_text = temp_dir.clone();
	some_text.push("texts");
	fs::create_dir(&some_text).unwrap();
	some_text.push("some_text");
	fs::write(&some_text, b"Some text").unwrap();
	
	CacheBust::builder()
		.in_dir(temp_dir)
		.in_place(true)
		.build()
		.execute().unwrap();
	
	empty.set_file_name("empty-e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
	some_text.set_file_name("some_text-4c2e9e6da31a64c70623619c449a040968cdbea85945bf384fa30ed2d5d24fa3");
	
	assert_eq!(fs::read(empty).unwrap(), b"");
	assert_eq!(fs::read(some_text).unwrap(), b"Some text");
}

#[test]
fn in_place_with_out_dir() {
	let temp_dir = create_temp_dir("in_place_with_out_dir");
	
	let mut empty = temp_dir.clone();
	empty.push("empty");
	File::create(&empty).unwrap();
	
	let mut some_text = temp_dir.clone();
	some_text.push("texts");
	fs::create_dir(&some_text).unwrap();
	some_text.push("some_text");
	fs::write(&some_text, b"Some text").unwrap();
	
	CacheBust::builder()
		.in_dir(temp_dir)
		.out_dir(create_temp_dir("in_place_with_out_dir2"))
		.in_place(true)
		.build()
		.execute().unwrap();
	
	empty.set_file_name("empty-e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
	some_text.set_file_name("some_text-4c2e9e6da31a64c70623619c449a040968cdbea85945bf384fa30ed2d5d24fa3");
	
	assert_eq!(fs::read(empty).unwrap(), b"");
	assert_eq!(fs::read(some_text).unwrap(), b"Some text");
}

#[test]
#[should_panic(expected = "out_dir must be specified or in_place set to true")]
fn no_out_dir() {
	CacheBust::builder()
		.build();
}

#[test]
#[should_panic(expected = "is not a directory")]
fn in_dir_not_a_dir() {
	let mut file = assets_dir();
	file.push("hello.txt");
	
	CacheBust::builder()
		.in_dir(file)
		.in_place(true)
		.build();
}
