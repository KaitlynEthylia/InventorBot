use std::path::PathBuf;

use clap::Parser;
use anyhow::{Context, Result};
use log::{Level, LevelFilter};
use simplelog::{
	Color, ColorChoice::Auto, ConfigBuilder, LevelPadding,
	TermLogger, TerminalMode,
};

macro_rules! env_prefix {
	($lit:literal) => {
		concat!("INVENTOR_BOT_", $lit)
	};
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Command {
	#[arg(
		short, long,
		env = env_prefix!("CONFIG_FILE"),
		value_name = "FILE",
		help = "Override the default config file path.",
		default_value = crate::data::default_config_path().into_os_string(),
	)]
	pub config: PathBuf,

	#[arg(
		long,
		env = env_prefix!("CACHE_DIR"),
		value_name = "FILE",
		help = "Override the default cache directory path.",
		long_help = "Override the default cache directory path. \
			Takes precedent over that set in the config file.",
	)]
	pub cache: Option<PathBuf>,

	#[arg(
		short, long,
		env = env_prefix!("LOG"),
		value_name = "LOG_LEVEL",
		help = "Set the minimum level of logs to print.",
		long_help = "Set the minimum level of logs to print. Valid values are \
			'debug', 'info', 'warn', 'error', and 'off'.",
		value_parser = clap::value_parser!(LevelFilter),
		default_value = "warn",
	)]
	pub log: LevelFilter,

	#[arg(
		short, long,
		env = env_prefix!("TOKEN"),
		value_name = "TOKEN",
		help = "The authorisation token for the bot to use.",
		long_help = "The authorisation token for the bot to use. This will override \
			whatever is in the cache unless `--no-cache` is also passed. The token \
			must have `write:statuses permissions.",
	)]
	pub token: Option<String>,

	#[arg(
		short, long,
		env = env_prefix!("NO_CACHE"),
		help = "Disables caching.",
		long_help = "Disables caching, meaning that no authorisation tokens will be stored. \
			Useful if passing a token via the --token argument, and you don't want it to \
			override the currently stored token.",
	)]
	pub no_cache: bool,

	#[arg(
		short, long,
		env = env_prefix!("DRY_RUN"),
		help = "Don't actually try and post a status, just print the resulting message.",
	)]
	pub dry_run: bool,
}

pub fn init() -> Result<Command> {
	let args = Command::try_parse()?;

	TermLogger::init(
		args.log,
		ConfigBuilder::new()
			.set_time_level(LevelFilter::Trace)
			.set_level_color(Level::Info, Some(Color::Cyan))
			.set_level_color(Level::Debug, Some(Color::Magenta))
			.set_level_padding(LevelPadding::Right)
			.build(),
		TerminalMode::Mixed,
		Auto,
	)
	.context("Failed to initialise logger.")?;

	Ok(args)
}
