use std::{sync::mpsc, time::Duration};

use anyhow::{Context, Result};
use api::post::{self, PostCfg};
use rand::distributions::Uniform;

use crate::data::{config::Config, secrets::Secrets};

mod api;
mod cli;
mod data;

fn main() -> Result<()> {
	// initialisation
	let command = cli::init()?;
	let has_token = command.token.is_some();
	let config = Config::get(&command)?;
	api::init()?;

	// authorisation
	let secrets = match (config.cache.enable, command.token) {
		(_, Some(token)) => Some(Secrets {
			new: true,
			client: None,
			token,
		}),
		(true, None) => Secrets::load(&config),
		(false, None) => None,
	};
	let secrets = match secrets {
		Some(secrets) => secrets,
		None => Secrets::new(&config)?,
	};
	secrets.verify(&config.instance)?;

	// exit handler
	let (send, recv) = mpsc::channel::<()>();
	if let Err(e) = ctrlc::set_handler(move || {
		let _ = send.send(());
	}) {
		log::error!("Failed to set Ctrl-C handler: {e}");
	}

	// start
	let mut cfg = PostCfg {
		config: &config,
		secrets: &secrets,
		rng: rand::thread_rng(),
		dist: Uniform::new(0, config.inventors.len()),
	};
	let delay = config.repeat.map(|r| Duration::from_secs(r * 60));
	loop {
		if let Err(e) = post::post(&mut cfg) {
			log::error!("{e}");
		}
		match delay {
			Some(delay) => {
				if recv.recv_timeout(delay).is_ok() {
					break;
				}
			},
			None => break,
		}
	}

	// shutdown
	match (config.cache.enable, secrets.new, has_token) {
		(true, true, _) => secrets.dump(&config).context("Failed to save cache.")?,
		(false, _, false) => secrets.revoke(&config.instance)?,
		_ => {}
	}

	Ok(())
}
