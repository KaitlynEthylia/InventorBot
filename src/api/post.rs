use anyhow::Result;
use rand::{distributions::Distribution, Rng};
use serde::Serialize;

use crate::data::{config::Config, secrets::Secrets};

pub struct PostCfg<'cfg, R, D>
where
	R: Rng,
	D: Distribution<usize>,
{
	pub config: &'cfg Config,
	pub secrets: &'cfg Secrets,
	pub rng: R,
	pub dist: D,
}

pub fn post<T, D>(cfg: &mut PostCfg<T, D>) -> Result<()>
where
	T: Rng,
	D: Distribution<usize>,
{
	#[derive(Serialize)]
	struct Request<'a> {
		content_type: &'static str,
		visibility: &'a str,
		status: &'a str,
	}
	log::info!("Sending post");
	let inventor = &cfg.config.inventors[cfg.rng.sample(&cfg.dist)];
	let invention = super::words::gen_item()?;
	let status =
		&format!("I can't believe {inventor} invented {invention}");

	if cfg.config.dry_run {
		println!("{status}");
		return Ok(());
	}

	let params = Request {
		content_type: "text/plain",
		visibility: (&cfg.config.visibility).into(),
		status,
	};

	let url =
		format!("https://{}/api/v1/statuses", cfg.config.instance,);
	super::RQ_CLIENT
		.get()
		.unwrap()
		.post(url)
		.form(&params)
		.header("Authorization", &cfg.secrets.token)
		.header("Idempotency-Key", status)
		.send()?
		.error_for_status()?;

	Ok(())
}
