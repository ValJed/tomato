# tomato <img src="./docs/tomato.svg" alt="drawing" width="25"/>

Tomato is a simple pomodoro TUI. 

Designed to be flexible allowing to choose if you need a break or not if you're in you flow state. 

It'll also allow you to modify the duration of your session when starting it.

## Install

You can get the binary directly from each release or build from source:

```bash
cargo build --release
```

## Default durations

At first startup it'll create a config file located in `~/.config/tomato/config.toml` with default value that you can change.

```toml
default_work_duration = 25
default_break_duration = 5
```

## History

It does not contain work history for now.
