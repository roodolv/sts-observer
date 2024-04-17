# sts-observer

## About

It is a simple resident application that monitors the autosave files of the game 'Slay the Spire' and automatically outputs a txt file. This software was developed for the purpose of learning Rust.

And this software uses the 'libsts' crate, so please read the documents below to modify the exported txt files.

- [libsts API Documentation](https://docs.rs/libsts/)
- Cargo package: [libsts](https://crates.io/crates/libsts)

## How to build

### Using Terminal

```bash
git clone https://github.com/roodolv/sts-observer.git
cd sts-observer
cargo build --release
cp -f ./settings.json ./target/release/settings.json
```

## Usage

NOTE: Please place the JSON file ("settings.json") in the same directory as the executable.

Please execute the file created following the above instructions.

Alternatively, DOWNLOAD the executable from here:
[Latest Releases](https://github.com/roodolv/sts-observer/releases/latest)

## Thanks
- [Mega Crit](https://www.megacrit.com/)
    - The official website of the creator of the game
- [dcchut/libsts](https://github.com/dcchut/libsts)
    - GitHub repository of the useful sts library(crate)

## License
This software is licensed under the MIT license.
- MIT license: [LICENSE](LICENSE) or https://opensource.org/license/mit
