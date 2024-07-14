pub mod cache;
pub mod config;
pub mod secrets;

use std::{
	fs::OpenOptions,
	path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub fn default_config_path() -> PathBuf {
	dirs::config_dir()
		.expect("Could not find anywhere to store config file.")
		.join("inventor_bot.toml")
}

pub fn default_cache_path() -> PathBuf {
	dirs::cache_dir()
		.expect("Could not find anywhere to store cache.")
		.join("inventor_bot")
}

pub fn exists(path: impl AsRef<Path>) -> Result<bool> {
	path.as_ref()
		.try_exists()
		.context("File data unavailable.")
}

pub fn is_valid(path: impl AsRef<Path>, secure: bool) -> Result<()> {
	let mut opts = OpenOptions::new();
	opts.write(true);

	#[cfg(unix)]
	if secure {
		use std::os::unix::fs::OpenOptionsExt;
		opts.mode(600);
		// is there a windows equivalent to this?
	}

	opts.open(path)?;
	Ok(())
}
