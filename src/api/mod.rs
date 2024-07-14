use std::{sync::OnceLock, time::Duration};

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

pub mod auth;
pub mod words;
pub mod post;

#[derive(Deserialize)]
struct Error<'a> {
	error: &'a str,
	error_description: Option<&'a str>,
}

static RQ_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init() -> Result<()> {
	let client = Client::builder()
		.user_agent("inventor_bot")
		.timeout(Duration::from_secs(30))
		.build()
		.context("Failed to construct http client.")?;
	RQ_CLIENT.set(client).unwrap();
	Ok(())
}
