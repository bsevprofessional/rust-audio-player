/*
Result<()> is Rust’s “this might fail” return type. Throwable?
Context attaches error messages.
*/
use anyhow::{Context, Result};

/*
Device Section
*/
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{OutputStream, Sink};

use std::path::PathBuf;

mod audio;
mod files;
mod playback;
mod types;
mod ui;

use playback::Playback;
use types::Action;

use ui::{file_picker, menu_select, pick_device, TerminalMode};

fn main() -> Result<()> {
    let _term = TerminalMode::new()?;

    let host = rodio::cpal::default_host();
    let devices: Vec<_> = host
        .devices()
        .context("Failed to get devices")?
        .filter(|d| d.default_output_config().is_ok())
        .collect();

    if devices.is_empty() {
        anyhow::bail!("No audio output devices found");
    }

    let device = pick_device(&devices)?;
    let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());

    let (_stream, handle) =
        OutputStream::try_from_device(device).context("Failed to open device")?;

    let test_dir = PathBuf::from("../test_files/");
    if !test_dir.exists() {
        anyhow::bail!(
            "Directory {:?} does not exist. Create it and add audio files.",
            test_dir
        );
    }

    let mut sink: Option<Sink> = None;
    let mut current_file: Option<PathBuf> = None;
    let mut playback = Playback::new();

    let mut volume: f32 = 1.0;

    loop {
        let action = menu_select(
            &device_name,
            &test_dir,
            &mut current_file,
            &mut sink,
            &mut playback,
            volume,
        )?;


        match action {
            Action::SelectFile => {
                if let Some(path) = file_picker(&test_dir)? {
                    audio::play_file(&path, &handle, &mut sink, &mut playback)?;
                    current_file = Some(path);

                    if let Some(s) = sink.as_ref() {
                        s.set_volume(volume);
                    }
                }
            }
            Action::PauseResume => {
                if let Some(s) = sink.as_ref() {
                    if s.is_paused() {
                        s.play();
                        playback.resume();
                    } else {
                        s.pause();
                        playback.pause();
                    }
                }
            }
            Action::Stop => {
                if let Some(s) = sink.take() {
                    s.stop();
                }
                playback.stop();
                current_file = None;
            }
            Action::VolumeUp => {
                volume = (volume + 0.1).min(2.0);
                if let Some(s) = sink.as_ref() {
                    s.set_volume(volume);
                }
            }
            Action::VolumeDown => {
                volume = (volume - 0.1).max(0.0);
                if let Some(s) = sink.as_ref() {
                    s.set_volume(volume);
                }
            }
            Action::Quit => {
                if let Some(s) = sink.take() {
                    s.stop();
                }
                break;
            }
        }
    }

    Ok(())
}
