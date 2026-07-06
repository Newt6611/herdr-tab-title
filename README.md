# Herdr Tab Title

Automatically formats Herdr tab titles with their workspace-local position.

By default, tabs are renamed like this:

```text
1. Codex
2. Terminal
3. Notes
```

The default format is:

```text
{index}. {title}
```

## Requirements

- Herdr `0.7.0` or newer
- Rust toolchain with `cargo`

## Install

Install the plugin from GitHub:

```sh
herdr plugin install Newt6611/herdr-tab-title
```

Confirm Herdr sees it:

```sh
herdr plugin list
```

You should see `herdr-tab-title` listed as enabled.

## Install From Local Checkout

Use this flow when developing the plugin locally.

Clone or open this repository, then build the release binary:

```sh
cargo build --release
```

Link the plugin into Herdr:

```sh
herdr plugin link /path/to/herdr-tab-title
```

For example, if this repo is at `/Users/newt/dev/herdr-tab-title`:

```sh
herdr plugin link /Users/newt/dev/herdr-tab-title
```

Confirm Herdr sees it:

```sh
herdr plugin list
```

You should see `herdr-tab-title` listed as enabled and local.

## Configure

Configuration is optional. If there is no config file, the plugin uses:

```toml
format = "{index}. {title}"
```

To customize it, copy the example config into Herdr's plugin config directory.
You can find that path with:

```sh
herdr plugin list
```

The plugin entry includes a `config:` path. Copy the example file there as
`config.toml`:

```sh
cp config.example.toml /path/from/plugin-list/config.toml
```

Then edit `config.toml`.

Supported placeholders:

- `{index}`: the tab's 1-based position inside its workspace
- `{title}`: the tab title with any existing numeric prefix removed

Example:

```toml
format = "[{index}] {title}"
```

## Run Manually

The plugin updates titles automatically when tabs or workspaces change. You can also run it manually from Herdr with the `Refresh tab titles` plugin action.

## Update

After pulling changes, rebuild the binary:

```sh
cargo build --release
```

If the plugin is already linked from this directory, Herdr will continue using the rebuilt binary. You can relink if you want to refresh the registration:

```sh
herdr plugin link /path/to/herdr-tab-title
```

## Troubleshooting

If the plugin does not appear in Herdr, run:

```sh
herdr plugin list
```

If the binary is missing, rebuild:

```sh
cargo build --release
```

If a custom format is not applied, check that `config.toml` is in the config directory shown by `herdr plugin list`, and that the format only uses supported placeholders.
