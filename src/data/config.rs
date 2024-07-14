use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use toml;

use crate::cli::Command;

fn default_lang() -> String { String::from("en") }
fn default_client_name() -> String { String::from("inventor_bot") }
const fn default_port() -> u16 { 65233 }

#[derive(Debug, Deserialize)]
pub struct Config {
	pub instance: String,
	#[serde(default)]
	pub inventors: Vec<String>,
	pub repeat: Option<u64>,
	#[serde(default)]
	pub visibility: Visibility,

	pub cache: Cache,

	#[serde(default = "default_port")]
	pub port: u16,
	#[serde(default = "default_client_name")]
	pub client: String,
	#[serde(default = "default_lang")]
	pub lang: String,
	#[serde(default)]
	pub dry_run: bool,
}

#[derive(Debug, Deserialize, Default)]
pub enum Visibility {
	Public,
	#[default]
	Unlisted,
}

impl Into<&str> for &Visibility {
    fn into(self) -> &'static str {
		match self {
			&Visibility::Public => "public",
			&Visibility::Unlisted => "unlisted",
		}
    }
}

#[derive(Debug, Deserialize)]
pub struct Cache {
	#[serde(default)]
	pub enable: bool,
	#[serde(default = "super::default_cache_path")]
	pub path: PathBuf,
	#[serde(default)]
	pub protect: bool,
}

impl Config {
	pub fn get(command: &Command) -> Result<Self> {
		let path = &command.config;
		super::is_valid(path, false).context(format!(
			"Config file at {:?} is not valid. \
			Invention Bot requires a config file in order to run. \
			Consult the documentation in order to see how to create one.",
			path
		))?;
		let file = fs::read_to_string(path).context(format!(
			"Failed to read config file at {:?}.",
			path
		))?;
		let mut config = toml::de::from_str::<Self>(&file)
			.context("Failed to parse config file.")?;

		if let Some(cache_dir) = &command.cache {
			config.cache.path = cache_dir.to_path_buf();
		};
		config.cache.enable = config.cache.enable && !command.no_cache;
		config.dry_run = config.dry_run || command.dry_run;

		if config.cache.enable && !config.cache.protect {
			log::warn!("You have chosen to cache the authorisation token, \
				but you have not chosen to encrypt it. \
				Make sure to keep your cache directory safe")
		}

		Ok(config)
	}
}
