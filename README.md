<div align="center">

[Installation](#installation) |
[Usage](#usage) |
[Configuration](#configuration) |
[Example](#example) |
[Contributing](#contributing)

# Inventor Bot

[![Github](https://img.shields.io/badge/Github-KaitlynEthylia%2FInventorBot-cec2fc?logo=github&style=for-the-badge)](https://github.com/KaitlynEthylia/InventorBot)
[![Crates.io](https://img.shields.io/crates/v/inventor_bot?color=%23f7b679&logo=rust&style=for-the-badge)](https://crates.io/crates/inventor_bot)
[![OQL License](https://img.shields.io/badge/%E2%9A%A7%EF%B8%8F%20License-OQL%201.1-bfdfff?style=for-the-badge)](https://oql.avris.it/license/v1.1)

A fedi bot for posts in the format of
'I can't believe __ invented __'.

</div>

<a id="installation" />

## Installation

Inventor Bot can be installed from crates.io or manually installed from
GitHub.

### Crates.io

Assuming you already have the rust toolchain installed, you can simply
install Inventor Bot by running

```sh
cargo install inventor_bot
```

### Nix

You can also build Inventor Bot using Nix flakes;

```sh
nix build github:KaitlynEthylia/InventorBot
```

<a id="usage" />

## Usage

Running the command inventor_bot will start the bot, but it will be
unable to run until you provide it with a
[configuration file](#configuration). A few command line arguments
are provided to modify how the bot runs:


**-c, --config \<FILE>**: Override the default config file path.
<br />

**--cache \<DIR>**: Override the default cache directory path,
ignoring any value set in the config.
<br/ >

**-l, --log \<LEVEL>**: Set the minimum level of logs to print. Valid
values are 'debug', 'info', 'warn', 'error', and 'off'.
<br />

**-t, --token \<TOKEN>**: The authorisation token for the bot to use.
This will override whatever is in the cache unless `--no-cache` is
also passed. The token must have `write:statuses` permissions.
<br />

**-n, --no-cache**: Disables caching, meaning that no authorisation
tokens will be stored. Useful if passing a token via the --token
argument, and you don't want it to override the currently stored
token.

<a id="configuration" />

## Configuration

By default, Inventor Bot will look for configuration in the following places:

| Platform | Location                                             |
|----------|------------------------------------------------------|
|  Linux   | `$XDG_CONFIG_HOME/inventor_bot.toml`                 |
| Windows  | `%APPDATA%\inventor_bot.toml`                        |
|  MacOS   | `$HOME/Library/Application Suport/inventor_bot.toml` |

The config is relatively small, with only a few important options:

| Option | Type | Description |
|---|---|---|
| `inventors` | List of strings | Possible people to fill in the first blank in "I can't believe ___ invented ___". |
| `instance` | String | The Fedi instance to post to |
| `repeat` | Null or Integer | The delay between repeated postings in minutes. Null (i.e. Omitting the option) means that the bot will make a post and then the application will exit.
| `visibility` | `"public"` or `"unlisted"` | The visibility of the posts made to fedi.


### Caching

To prevent you from having to log in every time you start the bot, It's possible to cache the authorisation token you generate when logging in. The following options are available for caching:

| Option | Type | Description |
|---|---|---|
| `cache.enable` | Boolean | Whether to enable caching, off by default. |
| `cache.path` | String | The path to store the cache at. This should be a directory. The default value on linux is `$XDG_CACHE_HOME/inventor_bot/` |
| `cache.protect` | Boolean | Whether to encrypt the cache with a password. If caching is enabled and this is not, a warning will be printed whenever you start the bot. |

### Advanced Options

These options are not particularly useful to most people, but exist anyways:

| Option | Type | Description |
|---|---|---|
| `dry_run` | Boolean | When enabled, the bot will never actually post, and print to the terminal instead. This exists mostly to help with debugging. |
| `port` | Integer | The port to listen on whilst waiting for authorisation |
| `lang` | String | The language to request the authorisation page be displayed in. Currently this has no effect on the output of the bot. If I can be bothered, I might change this. |
| `client` | String | The name of the client to register the bot under to the fedi instance |

<a id="example" />

## Example

An example config may look something like this:

```toml
instance = "tech.lgbt"
repeat = 360

inventors = [
	"Hatsune Miku",
	"Hatsune Miku",
	"Hatsune Miku",
	"Yassie",
	"Yassie",
	"Kasane Teto",
	"Inventor Bot",
	"your mum",
	"Luna",
	"jerma985",
	"the media",
]

[cache]
enable = true
protect = true
```

<a id="contributing" />

## Contributing

Like.. just open a PR or something, idk...
