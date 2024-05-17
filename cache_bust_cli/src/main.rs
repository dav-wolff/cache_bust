use std::{path::PathBuf, process};

use cache_bust::CacheBust;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cachebust", version, about)]
struct Args {
	/// The directory containing the files to be hashed
	source: PathBuf,
	/// The directory to write the hashed files to, leave empty to modify files in-place
	#[arg(short, long)]
	out: Option<PathBuf>,
	/// A single file to hash instead of the entire directory
	#[arg(short, long)]
	file: Option<PathBuf>,
	/// Prints the name of the hashed file to stdout, only works when --file is given, exclusive with --print-file-path
	#[arg(long)]
	print_file_name: bool,
	/// Prints the path of the hashed file to stdout, only works when --file is given, exclusive with --print-file-name
	#[arg(long)]
	print_file_path: bool,
}

fn main() {
	let args = Args::parse();
	
	if (args.print_file_name || args.print_file_path) && args.file.is_none() {
		eprintln!("[cache_bust/error] Options --print-file-name and --print-file-path can only be used in combination with --file");
		process::exit(1);
	}
	
	if args.print_file_name && args.print_file_path {
		eprintln!("[cache_bust/error] Options --print-file-name and --print-file-path are mutually exclusive");
		process::exit(1);
	}
	
	let mut builder = CacheBust::builder()
		.in_dir(args.source)
		.is_build_script(false)
		.enable_logging(!(args.print_file_name || args.print_file_path));
	
	if let Some(out) = args.out {
		builder = builder.out_dir(out);
	} else {
		builder = builder.in_place(true);
	}
	
	let cache_bust = match builder.try_build() {
		Ok(cache_bust) => cache_bust,
		Err(err) => {
			eprintln!("[cache_bust/error] Error: {err}");
			process::exit(1);
		},
	};
	
	match args.file {
		None => {
			cache_bust.hash_folder().unwrap_or_else(|err| {
				eprintln!("[cache_bust/error] An error occured:\n{err}");
				process::exit(1);
			});
		},
		Some(file) => {
			let path = cache_bust.hash_file(file).unwrap_or_else(|err| {
				eprintln!("[cache_bust/error] An error occured:\n{err}");
				process::exit(1);
			});
			
			if args.print_file_name || args.print_file_path {
				let result = if args.print_file_name {
					path.file_name().expect("File should have a name").to_str().map(ToOwned::to_owned)
				} else {
					path.canonicalize().expect("Path should be correct").to_str().map(ToOwned::to_owned)
				};
				
				match result {
					None => {
						eprintln!("[cache_bust/error] File path is unprintable: {path:?}");
						process::exit(1);
					},
					Some(line) => {
						println!("{line}");
						process::exit(0);
					},
				}
			}
		},
	}
	
	println!("[cache_bust/info] All done.")
}
