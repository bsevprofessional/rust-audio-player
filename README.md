# rust-audio-player

A simple terminal-based audio player written in Rust.

This player lets you:

- Select an audio output device
- Browse and play audio files
- Pause / resume / stop playback
- Control volume
- View playback progress

---

## Requirements

You need:

- Rust (https://www.rust-lang.org/tools/install)
- Cargo (comes with Rust)
- Linux / macOS / Windows with audio support

Supported audio formats depend on your system and enabled codecs, but typically include:

- mp3
- wav
- flac
- ogg
- m4a

---

## Setup

Make sure you have a folder for your audio files:

test_files/


Place your audio files inside this folder.

Example project layout:

rust-audio-player/
├── audio_player/
│ ├── Cargo.toml
│ └── src/
└── test_files/
├── song1.mp3
└── song2.wav


---

## How to Run

From the root of the repository, navigate to the player directory:

```bash
c   d audio_player
```
Then run:
```bash
    cargo run
```

Cargo will compile the project and start the player.
Usage
Select Output Device

When the program starts, you will see a list of available audio devices.

Use:

    ↑ / ↓ to move

    Enter to select

If you do not hear audio, try selecting pipewire or another system device.
Main Menu

After selecting a device, the main menu will appear.

Navigate using:

    ↑ / ↓

    Enter

Menu options change depending on playback state.

Available options include:

    Select file and play

    Pause / Resume

    Stop

    Volume +

    Volume -

    Quit

Select Audio File

Choose Select file and play to browse files in ../test_files/.

Use the arrow keys and Enter to select a file.

Press q or Esc to return to the main menu.
Playback Controls

While a song is playing:

    Pause / Resume — Toggle playback

    Stop — Stop current track

    Volume + / - — Adjust volume

    Quit — Exit the player

Current volume is displayed as a percentage in the header.
Progress Bar

During playback, the header shows playback progress:

Time   : 01:23 [######------] 03:45

This displays:

    Current playback time

    Visual progress bar

    Total track length

Keyboard Controls
Key	Action
↑ / ↓	Navigate menus
Enter	Select option
q / Esc	Back (file selection)
Ctrl + C	Force quit
Troubleshooting
No Sound

Try selecting a different output device when prompted (for example: pipewire instead of default).
No Files Found

Make sure your audio files are located in:

test_files/

relative to the audio_player/ directory.
Build Issues

If you encounter build errors, update Rust and rebuild:

rustup update
cargo clean
cargo run

Notes

    This project uses rodio for audio playback.

    Terminal UI is built using crossterm.

    Audio duration detection uses symphonia.

License

MIT License
Author Bruno Gomez-Severino

Created as a learning project for Rust audio and terminal UI development.