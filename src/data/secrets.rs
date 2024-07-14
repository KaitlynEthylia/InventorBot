use std::{
	fs::{self, File}, io::Read, path::Path, process, time::{SystemTime, UNIX_EPOCH}
};

use anyhow::{anyhow, Result};
use pgp::{
	crypto::{hash::HashAlgorithm, sym::SymmetricKeyAlgorithm},
	ser::Serialize as _,
	types::{CompressionAlgorithm, StringToKey},
	Deserializable, Message,
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use toml;

use super::{cache::Cache, config::Config};
use crate::api::auth::{self, Client};

#[derive(Debug, Serialize, Deserialize)]
pub struct Secrets {
	pub client: Option<Client>,
	pub token: String,
	#[serde(default, skip_serializing)]
	pub new: bool,
}

fn get_password() -> String {
	rpassword::prompt_password("Password: ").unwrap()
}

fn encrypt<'a>(content: impl AsRef<str>) -> Result<Vec<u8>> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
	let mut rng = StdRng::seed_from_u64(now);

	let encrypted = Message::new_literal("data", content.as_ref())
		.encrypt_with_password(
			&mut rng,
			StringToKey::Salted {
				hash_alg: HashAlgorithm::SHA2_256,
				salt: *b"boobies!",
			},
			SymmetricKeyAlgorithm::AES256,
			get_password,
		)?
		.compress(CompressionAlgorithm::ZLIB)?;

	Ok(encrypted.to_bytes()?)
}

fn decrypt(content: impl Read) -> Result<String> {
	let message = Message::from_bytes(content)?
		.decompress()?
		.decrypt_with_password(get_password)
		.map_err(|_| anyhow!("Failed to decrypt, aborting."))?
		.get_content()?
		.unwrap();
	let message = String::from_utf8(message)?;
	Ok(message)
}

impl Secrets {
	// GET /api/v1/apps/verify_credentials
	pub fn verify(&self, instance: impl AsRef<str>) -> Result<()> {
		auth::verify(&self.token, instance)
	}

	// POST /oauth/revoke
	pub fn revoke(self, instance: impl AsRef<str>) -> Result<()> {
		// Client will only be None if a token is passed via the command line,
		// In which case revoke is never called.
		self.client
			.expect("This should never happen")
			.revoke(self.token, instance)
	}

	pub fn new(config: &Config) -> Result<Self> {
		let client = Client::new(config)?;
		let token = client.token(config)?;
		Ok(Self {
			new: true,
			client: Some(client),
			token,
		})
	}

	fn get(path: impl AsRef<Path>, protect: bool) -> Result<Self> {
		let path = path.as_ref();
		super::is_valid(&path, true)?;
		let mut file = File::open(&path)?;
		let file = match protect {
			true => {
				decrypt(file).unwrap_or_else(|e| {
					log::error!("{e}");
					process::exit(1)
				})
			},
			false => {
				let mut data = String::new();
				file.read_to_string(&mut data)?;
				data
			},
		};
		Ok(toml::de::from_str::<Self>(&file)?)
	}

	pub fn load(config: &Config) -> Option<Self> {
		log::info!("Loading data cache");
		let cache =
			Cache::load(&config.cache.path.join("cache.toml"))?;
		if cache.instance != config.instance {
			log::warn!("Data cache is for instance {}, current instance is {}. Ignoring.",
				cache.instance, config.instance);
			return None;
		}
		let mut protect = config.cache.protect;
		match (cache.protect, config.cache.protect) {
			(true, false) => {
				log::warn!("Cache was saved with password, but config has dissabled password \
					protection. Ignoring cache.");
				return None;
			},
			(false, true) => {
				log::warn!("Cache was saved without password, but password protection is enabled. \
				Delete the existing cache in order to save a new one with a password.");
				protect = false;
			},
			_ => {},
		}
		let path = &config.cache.path.join("data");
		match super::exists(path) {
			Err(e) => {
				log::error!(
					"{}",
					e.context(
						"Could not verify existence of data cache. \
						Assuming non-existence."
					)
				);
				return None;
			},
			Ok(false) => return None,
			Ok(true) => {},
		};
		match Self::get(path, protect) {
			Ok(secrets) => Some(secrets),
			Err(e) => {
				log::error!("{e}");
				log::error!(
					"{}",
					e.context(
						"Could not load data cache. Continuing without."
					)
				);
				None
			},
		}
	}

	pub fn dump(self, config: &Config) -> Result<()> {
		log::info!("Storing data cache.");
		fs::create_dir_all(&config.cache.path)?;
		let path = &config.cache.path.join("data");
		let data = toml::to_string(&self)?;
		let data = if config.cache.protect {
			encrypt(data)?
		} else {
			data.into_bytes()
		};
		let res = fs::write(path, data);
		if res.is_ok() {
			Cache {
				instance: config.instance.clone(),
				protect: config.cache.protect,
			}
			.dump(&config.cache.path.join("cache.toml"))?;
		}
		Ok(res?)
	}
}
