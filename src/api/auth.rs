use std::{
	env,
	io::{Read, Write},
	net::{Shutdown, TcpListener},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::data::config::Config;

use super::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
	pub client_id: String,
	pub client_secret: String,
}

const SCOPES: &'static str = "write:statuses";
impl Client {
	// POST /api/v1/apps
	pub fn new(config: &Config) -> Result<Self> {
		#[derive(Serialize)]
		struct Request<'a> {
			client_name: &'a str,
			redirect_uris: &'a str,
			scopes: &'static str,
			website: &'static str,
		}
		log::info!("Requesting a new client session.");
		let rq_client = super::RQ_CLIENT.get().unwrap();
		let params = Request {
			client_name: &config.client,
			redirect_uris: &format!(
				"http://127.0.0.1:{}",
				config.port
			),
			scopes: SCOPES,
			website: env!("CARGO_PKG_REPOSITORY"),
		};
		let response = rq_client
			.post(format!("https://{}/api/v1/apps", config.instance))
			.form(&params)
			.send()?;
		let success = response.status().is_success();
		let text = response.text()?;
		if success {
			Ok(serde_json::from_str::<Client>(&text)?)
		} else {
			let err = serde_json::from_str::<super::Error>(&text)
				.map(|e| e.error)
				.unwrap_or(&text);
			Err(anyhow!("{}", err)
				.context("Failed to create client."))
		}
	}

	// GET /oauth/authorize
	fn authorise(&self, config: &Config) -> Result<String> {
		#[derive(Serialize)]
		struct Request<'a> {
			response_type: &'static str,
			client_id: &'a str,
			redirect_uri: &'a str,
			scope: &'static str,
			lang: &'a str,
		}
		log::info!("Authorising user");
		let params = Request {
			response_type: "code",
			client_id: &self.client_id,
			redirect_uri: &format!(
				"http://127.0.0.1:{}",
				config.port
			),
			scope: SCOPES,
			lang: &config.lang,
		};
		let url = format!(
			"https://{}/oauth/authorize?{}",
			config.instance,
			serde_qs::to_string(&params)?
		);
		if let Err(e) = open::that(&url) {
			let res = match env::var("BROWSER") {
				Ok(browser) => open::with(&url, browser),
				Err(e_) => {
					log::error!("{e_}");
					Err(e)
				},
			};
			if res.is_err() {
				log::error!(
					"{}",
					res.context("Could not open browser.")
						.unwrap_err()
				);
			}
		}
		println!(
			"Waiting for authentication...\n \
			If your browser does not open automatically, sign in at {url}"
		);

		let listener =
			TcpListener::bind(format!("127.0.0.1:{}", config.port))?;
		let (mut connection, _) = listener.accept()?;
		let mut buf = [0_u8; 256];
		connection.read(&mut buf)?;

		let mut headers = [httparse::EMPTY_HEADER; 32];
		let mut http = httparse::Request::new(&mut headers);
		http.parse(&buf)?;
		// this is hideous but rustfmt likes it for some reason
		connection.write_all(
			format!(
				"
				HTTP/1.1 303 See Other\r\
				Location: https://{}\r
				",
				config.instance
			)
			.trim()
			.as_bytes(),
		)?;
		if let Err(e) = connection.shutdown(Shutdown::Both) {
			log::warn!("{e}");
		}

		let path = http
			.path
			.ok_or(anyhow!("Did not receive authorisation code."))?;
		let start = path
			.find("?code=")
			.ok_or(anyhow!("Authorisation code is not present."))?
			+ 6;
		Ok(path
			.get(start..)
			.ok_or(anyhow!("Authorisation code is empty."))?
			.to_owned())
	}

	// POST /oauth/token
	pub fn token(&self, config: &Config) -> Result<String> {
		#[derive(Serialize, Debug)]
		struct Request<'a> {
			grant_type: &'static str,
			code: &'a str,
			client_id: &'a str,
			client_secret: &'a str,
			redirect_uri: &'a str,
			scope: &'static str,
		}
		#[derive(Deserialize)]
		struct Response<'a> {
			access_token: &'a str,
		}
		let code = self.authorise(config)?;
		log::info!("Requesting a new token");

		let rq_client = super::RQ_CLIENT.get().unwrap();
		let params = Request {
			grant_type: "authorization_code",
			code: &code,
			client_id: &self.client_id,
			client_secret: &self.client_secret,
			redirect_uri: &format!(
				"http://127.0.0.1:{}",
				config.port
			),
			scope: SCOPES,
		};
		let response = rq_client
			.post(format!("https://{}/oauth/token", config.instance))
			.form(&params)
			.send()?;
		let success = response.status().is_success();
		let text = response.text()?;
		if success {
			let token =
				serde_json::from_str::<Response>(&text)?.access_token;
			Ok(format!("Bearer {token}"))
		} else {
			let err = serde_json::from_str::<super::Error>(&text)
				.unwrap_or(Error {
					error: &text,
					error_description: None,
				});
			if let Some(desc) = err.error_description {
				Err(anyhow!("{desc}").context(err.error.to_owned()))
			} else {
				Err(anyhow!("{}", err.error)
					.context("Failed to get authorisation token"))
			}
		}
	}

	// POST /oauth/revoke
	pub fn revoke(
		self,
		token: String,
		instance: impl AsRef<str>,
	) -> Result<()> {
		#[derive(Serialize)]
		struct Request {
			client_id: String,
			client_secret: String,
			token: String,
		}
		log::info!("Revoking authorisation token.");
		let rq_client = super::RQ_CLIENT.get().unwrap();
		let params = Request {
			client_id: self.client_id,
			client_secret: self.client_secret,
			token,
		};
		let response = rq_client
			.post(format!(
				"https://{}/oauth/revoke",
				instance.as_ref()
			))
			.form(&params)
			.send()?;
		if !response.status().is_success() {
			let text = response.text()?;
			let err = serde_json::from_str::<super::Error>(&text)
				.unwrap_or(Error {
					error: &text,
					error_description: None,
				});
			if let Some(desc) = err.error_description {
				Err(anyhow!("{desc}").context(err.error.to_owned()))
			} else {
				Err(anyhow!("{}", err.error)
					.context("Failed to get authorisation token"))
			}
		} else {
			Ok(())
		}
	}
}

// GET /api/v1/apps/verify_credentials
pub fn verify(
	token: impl AsRef<str>,
	instance: impl AsRef<str>,
) -> Result<()> {
	log::info!("Verifying authorisation token.");
	super::RQ_CLIENT
		.get()
		.unwrap()
		.get(format!(
			"https://{}/api/v1/apps/verify_credentials",
			instance.as_ref()
		))
		.header("Authorization", token.as_ref())
		.send()?
		.error_for_status()
		.context("Authorisation token failed verification.")?;
	Ok(())
}
