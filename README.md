# Lupusreginaβ

A general purpose discord bot using the [serenity](https://github.com/serenity-rs/serenity) library. 

## Commands

| Admin | General | Owner | Fun | Moderation |
| :---: | :---:  | :--: | :--: | :--: |
| setprefix | about  | info | eightball | ban |
| | help  | ping | | unban |
| | avatar | game  | | |
| | | online | | |
| | | idle | | |
| | | dnd | | |
| | | invisible | | |
| | | reset | | |


## Building

### Linux

#### Debug
* `git clone https://github.com/flat/Lupusregina-`
* `cargo build` or `cargo run`

#### Release
* `git clone https://github.com/flat/Lupusregina-`
* `cargo build --release` or `cargo run --release`

## Using

### Bot Token
lupusregina will automatically load a .env file in the current directory, BOT_TOKEN is required. RUST_LOG may also be set.

### Settings
TODO

### Linux
`~/.config/lupusreginaβ/settings.ini`

```ini
[General]
owner = xxxxxxxxxxxxxxx
```