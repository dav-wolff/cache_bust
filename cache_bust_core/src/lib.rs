use std::{ffi::OsString, fs::File, io::{self, Read}, path::Path};

use sha2::{digest::Output, Digest, Sha256};

pub fn hashed_file_name(path: &Path) -> Result<OsString, io::Error> {
	let file = File::open(path)?;
	let hash = hex::encode(hash_file(file)?);
	
	let mut file_name = path.file_stem().unwrap_or_default().to_owned();
	file_name.push("-");
	file_name.push(hash);
	
	if let Some(extension) = path.extension() {
		file_name.push(".");
		file_name.push(extension);
	}
	
	Ok(file_name)
}

fn hash_file(mut file: File) -> Result<Output<Sha256>, io::Error> {
	let mut data = Vec::new();
	file.read_to_end(&mut data)?;
	Ok(Sha256::digest(data))
}
