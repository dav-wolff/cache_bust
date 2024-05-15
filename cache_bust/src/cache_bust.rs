use std::{fs, io, path::PathBuf};

use cache_bust_core::hashed_file_name;
use walkdir::WalkDir;

fn warn_prefix(is_build_script: bool) -> &'static str {
	if is_build_script {
		"cargo::warn="
	} else {
		"[cache_bust/warning] "
	}
}

#[derive(Clone, Debug)]
pub struct CacheBustBuilder {
	in_dir: Option<PathBuf>,
	out_dir: Option<PathBuf>,
	in_place: bool,
	is_build_script: bool,
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
		}
	}
}

impl CacheBustBuilder {
	pub fn in_dir(mut self, path: PathBuf) -> Self {
		self.in_dir = Some(path);
		self
	}
	
	pub fn out_dir(mut self, path: PathBuf) -> Self {
		self.out_dir = Some(path);
		self
	}
	
	pub fn in_place(mut self, in_place: bool) -> Self {
		self.in_place = in_place;
		self
	}
	
	pub fn is_build_script(mut self, is_build_script: bool) -> Self {
		self.is_build_script = is_build_script;
		self
	}
	
	pub fn build(self) -> CacheBust {
		match self.try_build() {
			Ok(cache_bust) => cache_bust,
			Err(err) => panic!("{err}"),
		}
	}
	
	pub fn try_build(self) -> Result<CacheBust, String> {
		let Some(in_dir) = self.in_dir else {
			return Err("in_dir must be set".to_owned());
		};
		
		if !in_dir.is_dir() {
			return Err(format!("{in_dir:?} is not a directory"));
		}
		
		let out_dir = match (self.in_place, self.out_dir) {
			(true, None) => None,
			(true, Some(_)) => {
				println!("{}in_place is set to true, ignoring out_dir", warn_prefix(self.is_build_script));
				None
			},
			(false, Some(out_dir)) => Some(out_dir),
			(false, None) => return Err("out_dir must be specified or in_place set to true".to_owned()),
		};
		
		if let Some(out_dir) = &out_dir {
			if out_dir.is_file() {
				return Err(format!("{out_dir:?} is already a file"));
			}
		}
		
		Ok(CacheBust {
			in_dir,
			out_dir,
			is_build_script: self.is_build_script,
		})
	}
}

#[derive(Debug)]
pub struct CacheBust {
	in_dir: PathBuf,
	out_dir: Option<PathBuf>,
	is_build_script: bool,
}

impl CacheBust {
	pub fn builder() -> CacheBustBuilder {
		CacheBustBuilder::default()
	}
	
	pub fn execute(&self) -> Result<(), io::Error> {
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
			
			if self.is_build_script {
				println!("cargo::rerun-if-changed={}", entry.path().to_str()
					.unwrap_or_else(|| panic!("could not register a build-time dependency on {:?}", entry.path()))
				);
			}
			
			let hashed_file_name = hashed_file_name(entry.path())?;
			
			if let Some(mut dest) = self.out_dir.clone() {
				dest.extend(entry.path().components().skip(in_dir_components));
				dest.pop();
				fs::create_dir_all(&dest)?;
				dest.push(hashed_file_name);
				println!("[cache_bust/info] copying {:?} -> {dest:?}", entry.path());
				fs::copy(entry.path(), dest)?;
			} else {
				let new_path = entry.path().with_file_name(hashed_file_name);
				println!("[cache_bust/info] moving {:?} -> {new_path:?}", entry.path());
				fs::rename(entry.path(), new_path)?;
			}
		}
		
		Ok(())
	}
}
