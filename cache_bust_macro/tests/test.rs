use cache_bust_macro::*;

#[test]
fn test_macro() {
	assert_eq!(asset!("hello.txt"), "hello.d9014c4624844aa5bac314773d6b689ad467fa4e1d1a50a1b8a99d5a95f72ff5.txt");
}

#[test]
fn test_macro_nested_dir() {
	assert_eq!(asset!("greetings/hi.txt"), "greetings/hi.c01a4cfa25cb895cdd0bb25181ba9c1622e93895a6de6f533a7299f70d6b0cfb.txt");
}
