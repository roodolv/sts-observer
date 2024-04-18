# sts-observer

## About

It is a simple resident application that monitors the autosave files of the game 'Slay the Spire' and automatically outputs a txt file. This software was developed for the purpose of learning Rust.

And this software uses the 'libsts' crate, so please read the documents below to modify the exported txt files.

- [libsts API Documentation](https://docs.rs/libsts/)
- Cargo package: [libsts](https://crates.io/crates/libsts)

## Features

This application performs the following tasks:

- Monitors the latest `.autosave` files in the specified PATH in the JSON file.
    - The default autosave PATH is `C:\Program Files (x86)\Steam\steamapps\common\SlayTheSpire\saves\`.
- Detects file modifications by retrieving their UNIX timestamps.
- Outputs a `.txt` file containing parameters of de-obfuscated `.autosave` files to the specified PATH in the JSON file.
    - The default output PATH is `C:\Users\Default\Desktop\`.
- Automatically transitions between 3 modes (Waiting, Watching, and FileIO) using a finite state machine.

The JSON values can be easily changed, and if you have the Rust development environment installed on your machine, you can freely customize the implementation of the output functions.


## Build

### Using Terminal

```bash
git clone https://github.com/roodolv/sts-observer.git
cd sts-observer
cargo build --release
cp -f ./settings.json ./target/release/settings.json
```

## Usage

**NOTE**: If you're running this from a file explorer, please ensure that the JSON file ("settings.json") is placed in the same directory as the executable. However, if you're using a terminal, make sure it's in your current directory.

Please run the file created following the instructions above.

Alternatively, you can download and run the executable from here:
[Latest Releases](https://github.com/roodolv/sts-observer/releases/latest)

## Thanks
- [Mega Crit](https://www.megacrit.com/)
    - The official website of the creator of the game
- [dcchut/libsts](https://github.com/dcchut/libsts)
    - GitHub repository of the useful sts library(crate)

## License
This software is licensed under the MIT license.
- MIT license: [LICENSE](LICENSE) or https://opensource.org/license/mit
