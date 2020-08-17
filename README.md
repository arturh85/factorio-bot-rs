Factorio Bot
============

This project tries to be a bot platform for the game
[Factorio](https://www.factorio.com) (version 1.0) inspired by [factorio-bot](https://github.com/Windfisch/factorio-bot/)

Features:
- extract factorio.zip/tar.xz and symlink bridge mod
- start Factorio server and/or one or multiple clients (unrestricted) 
- read factorio recipes/entity prototypes/item prototypes/graphics
- read map contents by chunk for leaflet based Map View
- REST Interface for
  - walk player
  - read player inventories
  - read other inventories
  - place entities
  - find entities in radius
  - insert to inventory / remove from inventory
  - mine entities
  - craft recipes
  - place blueprints
  - cheating variants of above methods for quick tests
- Typescript based Bot Manager which can:
  - Build starter base with iron/copper/stone/coal burner-mining-drills and stone-furnaces
  - Start automation research & manually craft 10 `automation-science` to insert into `lab`
- should work on Win/Mac/Linux, not tested on Mac
- MIT licenced

## Youtube Videos

- [Factorio Bot 0.1: Research automation with 1 Bot in 12:33](https://youtu.be/6KXYuVDRZ-I) 

## Setup 

- Install [rust/cargo](https://rustup.rs/) and [nodejs/npm](https://nodejs.org/)
- Clone this repository
- Download [Factorio](https://www.factorio.com) as .zip or .tar.xz into workspace/ directory (don't use the headless version!)
- Change archive_path in Settings.toml to correct filename
- Start server & one client with good seed:

```
cargo run -- start -c 1 --seed 1785882545 
```

See cargo run -- --help for other options.
On Windows the first start needs Administrative Privileges to create the symlinks to the mod directory.

## Docker 

There is also a docker image which starts the server in headless mode.
You can then connect to this headless server from different hosts with 

```
cargo run -- start --server <server-ip>
``` 

## Contribute

Send me your Pull Requests :)

## Contact

Email: [arturh@arturh.de](mailto:arturh@arturh.de)
