use std::path::PathBuf;

use cache_bust::CacheBust;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cachebust", version, about)]
struct Args {
	/// The directory containing the files to be hashed
	source: PathBuf,
	/// The directory to write the hashed files to, leave empty to modify files in-place1
	#[arg(short, long)]
	out: Option<PathBuf>,
}

fn main() {
	let args = Args::parse();
	
	let mut builder = CacheBust::builder()
		.in_dir(args.source)
		.is_build_script(false);
	
	if let Some(out) = args.out {
		builder = builder.out_dir(out);
	} else {
		builder = builder.in_place(true);
	}
	
	let cache_bust = match builder.try_build() {
		Ok(cache_bust) => cache_bust,
		Err(err) => {
			eprintln!("[cache_bust/error] Error: {err}");
			return;
		},
	};
	
	let result = cache_bust.execute();
	
	if let Err(err) = result {
		eprintln!("[cache_bust/error] An error occured:\n{err}");
	} else {
		println!("[cache_bust/info] All done.")
	}
}
