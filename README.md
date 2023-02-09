# Studio CLI

Simple CLI for Livepeer Studio.

## Build

Install [Rust](https://www.rust-lang.org/tools/install) and run:

```bash
cargo build --release
```

## Usage

```bash
cp ./target/release/studio /usr/local/bin/studio
```

```bash
studio 
```

or 

```bash
cargo run
```

## Features
- List Streams, Assets, Tasks
- Create Streams
- Upload Assets
- Get playback info
- Playback Assets (ffplay required)
- Playback asset rendition (ffplay required)
- Push into regions (ffmpeg required)
- Track task status
- Admin functionalities (using admin token)

