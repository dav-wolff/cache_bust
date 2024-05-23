use std::{error::Error, fmt::{self, Display}, fs, io, path::{Path, PathBuf}};

use cache_bust_core::hashed_file_name;
use walkdir::WalkDir;

fn warn_prefix(is_build_script: bool) -> &'static str {
	if is_build_script {
		"cargo::warn="
	} else {
		"[cache_bust/warning] "
	}
}

/// Error from trying to build a [CacheBust]
#[derive(Debug)]
pub enum CacheBustBuilderError {
	/// The option `in_dir` was not set and couldn't default to the
	/// `assets` directory of the crate, because the `CARGO_MANIFEST_DIR`
	/// environment variable wasn't set.
	InDirNotSet,
	/// The given `in_dir` is not a directory.
	InDirNotADirectory(PathBuf),
	/// Neither `out_dir` nor `in_place` were set.
	OutDirNotSet,
	/// `out_dir` is a file.
	OutDirIsAFile(PathBuf),
}

impl Display for CacheBustBuilderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use CacheBustBuilderError::*;
		match self {
			InDirNotSet => write!(f, "in_dir must be set"),
			InDirNotADirectory(in_dir) => write!(f, "{in_dir:?} is not a directory"),
			OutDirNotSet => write!(f, "out_dir must be specified or in_place set to true"),
			OutDirIsAFile(out_dir) => write!(f, "{out_dir:?} is already a file"),
		}
	}
}

impl Error for CacheBustBuilderError {}

/// Builder for [CacheBust]
#[derive(Clone, Debug)]
pub struct CacheBustBuilder {
	in_dir: Option<PathBuf>,
	out_dir: Option<PathBuf>,
	in_place: bool,
	is_build_script: bool,
	enable_logging: bool,
}

impl Default for CacheBustBuilder {
	fn default() -> Self {
		let in_dir = std::env::var_os("CARGO_MANIFEST_DIR")
			.map(|manifest_dir| {
				let mut assets_dir: PathBuf = manifest_dir.into();
				assets_dir.push("assets");
				assets_dir
			});
		
		let is_build_script = in_dir.is_some();
		
		Self {
			in_dir,
			out_dir: None,
			in_place: false,
			is_build_script,
			enable_logging: true,
		}
	}
}

impl CacheBustBuilder {
	/// Specifies the source directory of the files to hash. This option is required.
	/// 
	/// # Default
	/// 
	/// Defaults to the `assets` directory of the crate if `CARGO_MANIFEST_DIR` is set.
	pub fn in_dir(mut self, path: impl Into<PathBuf>) -> Self {
		self.in_dir = Some(path.into());
		self
	}
	
	/// Specifies the directory the hashed files will be written to. This option
	/// has no effect if `in_place` is set. Either this option or `in_place` has
	/// to be set.
	pub fn out_dir(mut self, path: impl Into<PathBuf>) -> Self {
		self.out_dir = Some(path.into());
		self
	}
	
	/// Specifies whether to rename the files in-place instead of copying them
	/// to a new directory.
	/// 
	/// # Default
	/// 
	/// `false`
	pub fn in_place(mut self, in_place: bool) -> Self {
		self.in_place = in_place;
		self
	}
	
	/// Specifies whether the executing context is a `build.rs` build script.
	/// If so, the proper `cargo::rerun-if-changed` instructions will be emitted.
	/// 
	/// # Default
	/// 
	/// `true` if `CARGO_MANIFEST_DIR` is set, `false` otherwise
	pub fn is_build_script(mut self, is_build_script: bool) -> Self {
		self.is_build_script = is_build_script;
		self
	}
	
	/// Specifies whether logging should be enabled.
	/// 
	/// # Default
	/// 
	/// `true`
	pub fn enable_logging(mut self, enable_logging: bool) -> Self {
		self.enable_logging = enable_logging;
		self
	}
	
	/// Builds a [CacheBust] with the given options.
	/// 
	/// # Panics
	/// 
	/// Panics if the given options aren't valid.
	pub fn build(self) -> CacheBust {
		match self.try_build() {
			Ok(cache_bust) => cache_bust,
			Err(err) => panic!("{err}"),
		}
	}
	
	/// Builds a [CacheBust] with the given options.
	/// 
	/// # Errors
	/// 
	/// Errors if the given options aren't valid.
	pub fn try_build(self) -> Result<CacheBust, CacheBustBuilderError> {
		let Some(in_dir) = self.in_dir else {
			return Err(CacheBustBuilderError::InDirNotSet);
		};
		
		if !in_dir.is_dir() {
			return Err(CacheBustBuilderError::InDirNotADirectory(in_dir));
		}
		
		let out_dir = match (self.in_place, self.out_dir) {
			(true, None) => None,
			(true, Some(_)) => {
				println!("{}in_place is set to true, ignoring out_dir", warn_prefix(self.is_build_script));
				None
			},
			(false, Some(out_dir)) => Some(out_dir),
			(false, None) => return Err(CacheBustBuilderError::OutDirNotSet),
		};
		
		if let Some(out_dir) = &out_dir {
			if out_dir.is_file() {
				return Err(CacheBustBuilderError::OutDirIsAFile(out_dir.clone()));
			}
		}
		
		Ok(CacheBust {
			in_dir,
			out_dir,
			is_build_script: self.is_build_script,
			enable_logging: self.enable_logging,
		})
	}
}

macro_rules! log {
	($do_log: expr, $($msg: tt)*) => {
		if $do_log {
			println!($($msg)*);
		}
	};
}

/// Struct for adding hashes to file names.
#[derive(Debug)]
pub struct CacheBust {
	in_dir: PathBuf,
	out_dir: Option<PathBuf>,
	is_build_script: bool,
	enable_logging: bool,
}

impl CacheBust {
	/// Returns a new [CacheBustBuilder] with default settings.
	pub fn builder() -> CacheBustBuilder {
		CacheBustBuilder::default()
	}
	
	/// Hashes all the files in the directory set by `in_dir` and either renames them
	/// to include their hashes if `in_place` is set, or copies them to `out_dir` with
	/// their hashes added to their names.
	/// 
	/// If `is_build_script` is set this emits the proper `cargo::rerun-if-changed` instruction.  
	/// If `enable_logging` is set this will print out a message for every moved file.
	pub fn hash_dir(&self) -> Result<(), io::Error> {
		if self.is_build_script {
			println!("cargo::rerun-if-changed={}", self.in_dir.to_str()
				.unwrap_or_else(|| panic!("could not register a build-time dependency on {:?}", self.in_dir))
			);
		}
		
		if let Some(out_dir) = &self.out_dir {
			if out_dir.is_dir() {
				fs::remove_dir_all(out_dir)?;
			}
		}
		
		let in_dir_components = self.in_dir.components().count();
		
		for entry in WalkDir::new(&self.in_dir) {
			let entry = entry?;
			
			if !entry.path().is_file() {
				continue;
			}
			
			let hashed_file_name = hashed_file_name(entry.path())?;
			
			if let Some(mut dest) = self.out_dir.clone() {
				dest.extend(entry.path().components().skip(in_dir_components));
				dest.pop();
				fs::create_dir_all(&dest)?;
				dest.push(hashed_file_name);
				log!(self.enable_logging, "[cache_bust/info] copying {:?} -> {dest:?}", entry.path());
				fs::copy(entry.path(), dest)?;
			} else {
				let new_path = entry.path().with_file_name(hashed_file_name);
				log!(self.enable_logging, "[cache_bust/info] moving {:?} -> {new_path:?}", entry.path());
				fs::rename(entry.path(), new_path)?;
			}
		}
		
		Ok(())
	}
	
	/// Hashes the specified file and either renames it to include its hash if `in_place` is set,
	/// or copies it to `out_dir` with the hash added to its name.
	/// 
	/// If `file` is a relative path then it is relative to `in_dir` and the structure of subdirectories
	/// containing the file is kept when copying it to `out_dir`.
	pub fn hash_file(&self, file: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
		let file = file.as_ref();
		
		let path = self.in_dir.join(file);
		
		if self.is_build_script {
			println!("cargo::rerun-if-changed={}", path.to_str()
				.unwrap_or_else(|| panic!("could not register a build-time dependency on {:?}", path))
			);
		}
		
		let hashed_file_name = hashed_file_name(&path)?;
		
		let dest = if let Some(mut dest) = self.out_dir.clone() {
			if file.is_relative() {
				dest.push(file);
				dest.pop();
			}
			
			fs::create_dir_all(&dest)?;
			dest.push(hashed_file_name);
			log!(self.enable_logging, "[cache_bust/info] copying {path:?} -> {dest:?}");
			fs::copy(path, &dest)?;
			dest
		} else {
			let new_path = path.with_file_name(hashed_file_name);
			log!(self.enable_logging, "[cache_bust/info] moving {path:?} -> {new_path:?}");
			fs::rename(path, &new_path)?;
			new_path
		};
		
		Ok(dest)
	}
}
