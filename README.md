<div align="center">
  <pre>
,--------.,--.   ,--.,------.,--.   ,--.  
'--.  .--' \  `.'  / |  .--. '\  `.'  /  
   |  |     '.    /  |  '--' | '.    /   
   |  |       |  |   |  | --'    |  |    
   `--'       `--'   `--'        `--'    
  </pre>
</div>

> [!WARNING]  
> When the terminal is too small it can lead to strange behavior.

## Table of contents
- [Overview](#overview)
- [Installation](#installation)
- [Flags](#flags)
- [Configuration](#configuration)
- [Stats](#stats)
- [Uninstall](#uninstall)

## Overview
![Description of the GIF](./docs/assets/demo.gif)
I wanted to create a simple typing game to improve my typing speed and accuracy. I really like using [monkeytype](https://monkeytype.com/) and I thought, why not create something similar
in the terminal? I searched for some but didn't find anything I really liked, so I built it myself. Typy is a terminal-based typing game that displays a random
word and asks you to type it as fast as possible. The game tracks your typing speed and accuracy, allowing you to monitor your progress over time. Typy also supports
different game modes, such as uppercase and punctuation, to help you improve your typing skills in different areas.

## Installation
To install Typy, you can use the [Cargo] package manager:

[Cargo]: https://doc.rust-lang.org/cargo/

```bash
cargo install --git "https://github.com/Pazl27/typy-cli.git" --tag "v0.9.0"
```

If you prefer to get the newest version and compile it yourself, follow these steps:

1. Clone the Typy repository:
    ```bash
    git clone https://github.com/Pazl27/typy-cli.git
    cd typy-cli
    ```

2. Compile the project:
    ```bash
    cargo build --release
    ```

3. Move the compiled binary to a directory in your PATH:
    ```bash
    sudo mv target/release/typy /usr/local/bin/
    ```

4. Ensure the `english.txt` file is in the correct location:
    ```bash
    mkdir -p ~/.local/share/typy
    cp resources/english.txt ~/.local/share/typy/
    ```

If you have Nix with flakes enabled, you can install typy-cli directly:

```bash
nix profile install github:Pazl27/typy-cli
```

Or to run without installing:

```bash
nix run github:Pazl27/typy-cli
```

## Home screen
Running `typy` with **no arguments** opens an interactive home screen (ASCII art + menu). From there you can:

- pick the test type (Time / Words / Zen), the time or word count, and toggle modes (uppercase, punctuation, numbers, quotes);
- press `Enter` to start, `s` to open the **settings menu**, `a` to open the **stats screen**, `q` to quit.

Navigation works with the arrow keys **or the vim keys** `h`/`j`/`k`/`l` on every menu.

Passing any test flag (`-t`, `-w`, `--zen`, `-m`) starts a test directly and skips the home screen.

### Settings menu (`s`)
Edit every setting in-app: theme/graph colors, cursor style, modes and their chances, the **keybind preset** (`10ff` ↔ `monkeytype`), and the **language** — which is picked from the wordlists (`*.txt` files) installed in `~/.local/share/typy`. Changes are **saved to `config.toml`** when you leave the menu (`Esc`). Use `↑↓`/`jk` to move, `←→`/`hl` to change a value, `Enter` to edit a text/hex field.

### Stats screen (`a` or `-s`)
A navigable table of every recorded test (date, WPM, RAW, accuracy, consistency, mode) with the rolling averages on top and the **WPM curve of the selected test** on the side. Press `d` to **delete** a test recorded by mistake — its contribution is removed from the averages too. Trivial runs (fewer than 3 typed words) are never recorded in the first place.

## Flags
The `Typy` application supports the following flags:

- `-t, --time <duration>`: Runs a timed test directly (in seconds).
  - if you set the time to a too low value the graph ends up scuffed.
  - e.g., `typy-cli -t 60` sets the game duration to 60 seconds.

- `-s, --stats`: Opens the detailed stats screen.
  - e.g., `typy-cli --stats` displays the game statistics.

- `-c, --config`: Creates a config file if it doesn't exist and opens it in `$EDITOR`.
  - e.g., `typy-cli --config` creates and opens the configuration file.

- `-m, --mode <mode>`: Sets the mode of the game. Multiple values can be specified.
  - possible modes are `normal`, `uppercase`, `punctuation`, `numbers` and `quotes`.
  - e.g., `typy-cli -m uppercase,punctuation` sets the game mode to uppercase and punctuation.

- `-w, --words <n>`: Runs a word-count test instead of a timed one.
  - e.g., `typy-cli -w 25` ends the test after 25 words.

- `--zen`: Zen mode — no time or word limit; type freely until you restart or quit.

## Controls
While typing:

- **Backspace**: delete the last character.
- **Ctrl + Backspace**: delete the current word.
- **Restart**: start a fresh test at any time. The gesture depends on the keybind preset:
  - `10ff` preset (default): press `Tab` then `Space`.
  - `monkeytype` preset: press `Tab` then `Enter`.
  - The **same gesture works on the result screen** after a test, to immediately start another one.
- **Esc** or **Ctrl + C**: quit.

The text scrolls automatically as you type, so tests are no longer limited to three lines.


## Configuration
Typy allows you to configure the colors (theme) via a TOML file. The configuration file is located at `~/.config/typy/config.toml`. You can also configure Typy using the command line with the `typy -c` option.
Inside of the configuration file, you can specify the colors for the theme, graph, and cursor style. Also you can specify some default settings.

Here is an example configuration block for the `config.toml` file:

```toml
# ~/.config/typy/config.toml

[theme]
fg = "#516D49"
missing = "#918273"
error = "#FB4934"
accent = "#D3869B"

[graph]
data = "#8EC07C"
title = "#458588"
axis = "#B16286"

[cursor]
style = "SteadyBar" # possible options are: DefaultUserShape, BlinkingBlock, SteadyBlock, BlinkingUnderScore, SteadyUnderScore, BlinkingBar, SteadyBar,

[modes]
default_mode = "normal" # possible modes are "normal"|"uppercase"|"punctuation"|"numbers"|"quotes", combinations of modes is also possible e.g: "uppercase, punctuation"
uppercase_chance = "3" # possible are values between 0 and 1, if value is too high it gets clamped to 1, if too low it gets clamped to 0
punctuation_chance = "0.5" # possible are values between 0 and 1, if value is too high it gets clamped to 1, if too low it gets clamped to 0
numbers_chance = "0.3" # chance a word is replaced by a random number in "numbers" mode (clamped between 0 and 1)

[keybinds]
preset = "10ff" # "10ff" (Tab then Space to restart) or "monkeytype" (Tab then Enter to restart)

[language]
lang = "english" # select your desired language
```

To apply the configuration, you can either edit the `config.toml` file directly or use the `typy -c` command to to open the file in your preferred editor:

```bash
typy -c 
```

This allows you to customize the appearance of Typy to match your preferences.

## Stats
Statistics are derived from the real per-keystroke timing of each test, which unlocks:

- **WPM** (net, correct characters only) and **RAW** (every keystroke).
- **Accuracy**: correct keystrokes over total keystrokes.
- **Consistency**: how steady your speed was across the test (based on the per-second WPM spread).
- A **WPM-over-time** graph and a **most-missed characters** breakdown on the finish screen.
- **Live WPM** shown in the header while you type.

The scores are saved in a file located at `~/.local/share/typy/scores.json`. The file tracks the last 10 games plus rolling averages.
To check your stats you can use the `typy --stats` command.

```bash
typy -s
```
This will display the stats of the last 10 games and looks something like this:
![Stats](./docs/assets/snapshot_2025-02-24_00-28-16.png)
To close this view press `Ctrl + c` or `esc`.

## Language
The language files are located at `~/.local/share/typy/`. The default language is `english`. You can change the language by editing the `config.toml` file or by using the
`typy -c` command. If you want to add a new language you can create a new file in the `~/.local/share/typy/` directory and add the words in the following format:
```txt
word1
word2
...
```
The language file should be named after the language you want to add. For example, if you want to add a German language file, you would create a file named `german.txt` and add the German words to it.
If you want to use the new language you need to change the `lang` field in the `config.toml` file to the name of the language file without the `.txt` extension.
If you want to provide a new language to the Typy repository, feel free to create a pull request. Atm I only have the `english.txt` file in the repository.

## Uninstall
```bash
cargo uninstall typy
```
