#![forbid(unsafe_code)]
#![deny(non_snake_case)]

use std::{path::PathBuf, process};

use cache_bust::CacheBust;
use clap::{Parser, ValueEnum};

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
	/// Prints either the hash, the name of the hashed file, or its path to stdout. Only works when --file is given
	#[arg(short, long)]
	print: Option<Print>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Print {
	Hash,
	FileName,
	FilePath,
}

fn main() {
	let args = Args::parse();
	
	if args.print.is_some() && args.file.is_none() {
		eprintln!("[cache_bust/error] Option --print can only be used in combination with --file");
		process::exit(1);
	}
	
	let mut builder = CacheBust::builder()
		.in_dir(args.source)
		.is_build_script(false)
		.enable_logging(args.print.is_none());
	
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
			cache_bust.hash_dir().unwrap_or_else(|err| {
				eprintln!("[cache_bust/error] An error occured: {err}");
				process::exit(1);
			});
		},
		Some(file) => {
			let path = cache_bust.hash_file(&file).unwrap_or_else(|err| {
				eprintln!("[cache_bust/error] An error occured: {err}");
				process::exit(1);
			});
			
			if let Some(print) = args.print {
				let result = match print {
					Print::FileName => path.file_name().expect("File should have a name").to_str().map(ToOwned::to_owned),
					Print::FilePath => path.canonicalize().expect("Path should be correct").to_str().map(ToOwned::to_owned),
					Print::Hash => {
						let stripped_path = if file.extension().is_some() {
							path.with_extension("")
						} else {
							path
						};
						
						let hash = stripped_path.extension().expect("File should have its hash as an extension")
							.to_str().map(ToOwned::to_owned).expect("Hash should be valid UTF-8");
						
						println!("{hash}");
						process::exit(0);
					},
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
