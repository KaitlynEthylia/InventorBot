use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
	pub instance: String,
	pub protect: bool,
}

impl Cache {
	fn get(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		super::is_valid(&path, true)?;
		let file = fs::read_to_string(&path)?;
		Ok(toml::de::from_str::<Self>(&file)?)
	}

	pub fn load(path: impl AsRef<Path>) -> Option<Self> {
		let path = path.as_ref();
		match super::exists(path) {
			Err(e) => {
				log::error!("{}", e.context("Could not verify existence of cache. Assuming non-existence."));
				return None;
			},
			Ok(false) => return None,
			Ok(true) => {},
		}
		match Self::get(path) {
			Ok(cache) => Some(cache),
			Err(e) => {
				log::error!(
					"{}",
					e.context(
						"Could not load cache. Continuing without."
					)
				);
				None
			},
		}
	}

	pub fn dump(self, path: impl AsRef<Path>) -> Result<()> {
		let data = toml::to_string(&self)?;
		fs::write(path.as_ref(), data)?;
		Ok(())
	}
}
