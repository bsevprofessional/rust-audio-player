use anyhow::{Context, Result};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, ClearType, DisableLineWrap, EnableLineWrap},
};
use rodio::cpal::traits::DeviceTrait;
use rodio::cpal::Device;
use rodio::Sink;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::files::list_audio_files;
use crate::playback::{format_time, progress_bar, Playback};
use crate::types::Action;

pub struct TerminalMode;

impl TerminalMode {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(std::io::stdout(), cursor::Hide, DisableLineWrap).ok();
        Ok(Self)
    }
}

impl Drop for TerminalMode {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), cursor::Show, EnableLineWrap);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn clear_screen() -> Result<()> {
    execute!(
        std::io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    Ok(())
}

pub fn ui_println(s: &str) -> Result<()> {
    execute!(std::io::stdout(), cursor::MoveToColumn(0))?;
    print!("{}", s);
    print!("\r\n");
    std::io::stdout().flush().ok();
    Ok(())
}

pub fn ui_blank() -> Result<()> {
    ui_println("")
}

pub fn read_key() -> Result<KeyCode> {
    loop {
        match event::read()? {
            Event::Key(KeyEvent { code, .. }) => return Ok(code),
            _ => {}
        }
    }
}

pub fn pick_device<'a>(devices: &'a Vec<Device>) -> Result<&'a Device> {
    let mut idx = 0;

    loop {
        clear_screen()?;
        ui_println("Select audio output device (↑/↓, Enter)")?;
        ui_blank()?;

        for (i, dev) in devices.iter().enumerate() {
            let name = dev.name().unwrap_or_else(|_| "Unknown".to_string());
            if i == idx {
                ui_println(&format!("> {}", name))?;
            } else {
                ui_println(&format!("  {}", name))?;
            }
        }

        match read_key()? {
            KeyCode::Up => {
                if idx > 0 {
                    idx -= 1;
                }
            }
            KeyCode::Down => {
                if idx + 1 < devices.len() {
                    idx += 1;
                }
            }
            KeyCode::Enter => return Ok(&devices[idx]),
            _ => {}
        }
    }
}

pub fn file_picker(dir: &Path) -> Result<Option<PathBuf>> {
    let files = list_audio_files(dir)?;
    if files.is_empty() {
        clear_screen()?;
        ui_println(&format!("No audio files found in {}", dir.display()))?;
        ui_blank()?;
        ui_println("Press 'q' to go back.")?;
        loop {
            if matches!(read_key()?, KeyCode::Char('q') | KeyCode::Esc) {
                return Ok(None);
            }
        }
    }

    let mut idx = 0;

    loop {
        clear_screen()?;
        ui_println("Select a file (↑/↓, Enter)  |  q/Esc = back")?;
        ui_blank()?;
        ui_println(&format!("Folder: {}", dir.display()))?;
        ui_blank()?;

        for (i, p) in files.iter().enumerate() {
            let name = p.file_name().unwrap().to_string_lossy();
            if i == idx {
                ui_println(&format!("> {}", name))?;
            } else {
                ui_println(&format!("  {}", name))?;
            }
        }

        match read_key()? {
            KeyCode::Up => {
                if idx > 0 {
                    idx -= 1;
                }
            }
            KeyCode::Down => {
                if idx + 1 < files.len() {
                    idx += 1;
                }
            }
            KeyCode::Enter => return Ok(Some(files[idx].clone())),
            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
            _ => {}
        }
    }
}

pub fn draw_header(
    device_name: &str,
    dir: &Path,
    current_file: &Option<PathBuf>,
    sink: &Option<Sink>,
    playback: &Playback,
    volume: f32,
) -> Result<()> {
    ui_println("Rust Audio Player")?;
    ui_blank()?;

    ui_println(&format!("Output : {}", device_name))?;
    ui_println(&format!("Folder : {}", dir.display()))?;

    let vol_pct = (volume * 100.0).round() as u32;
    ui_println(&format!("Volume : {}%", vol_pct))?;

    let state = match (current_file, sink) {
        (Some(p), Some(s)) => {
            let name = p.file_name().unwrap().to_string_lossy();
            if s.is_paused() {
                format!("paused  |  {}", name)
            } else {
                format!("playing |  {}", name)
            }
        }
        _ => "idle (no file selected)".to_string(),
    };

    ui_println(&format!("State  : {}", state))?;

    if current_file.is_some() && sink.is_some() {
        let elapsed = playback.elapsed();

        let bar = crate::playback::progress_bar(elapsed, playback.total, 30);

        let left = crate::playback::format_time(elapsed);
        let right = playback
            .total
            .map(crate::playback::format_time)
            .unwrap_or_else(|| "--:--".to_string());

        ui_println(&format!("Time   : {} {} {}", left, bar, right))?;
    }

    ui_blank()?;
    Ok(())
}


pub fn draw_menu(menu: &[(String, Action)], selected: usize) -> Result<()> {
    ui_println("Options (↑/↓, Enter):")?;
    ui_blank()?;

    for (i, (label, _)) in menu.iter().enumerate() {
        if i == selected {
            ui_println(&format!("> {}", label))?;
        } else {
            ui_println(&format!("  {}", label))?;
        }
    }
    Ok(())
}

pub fn menu_select(
    device_name: &str,
    dir: &Path,
    current_file: &mut Option<PathBuf>,
    sink: &mut Option<Sink>,
    playback: &mut Playback,
    volume: f32,
) -> Result<Action> {
    use crate::types::{build_menu, get_status};

    static mut LAST_IDX: usize = 0;
    let mut idx: usize = unsafe { LAST_IDX };

    loop {
        if let Some(s) = sink.as_ref() {
            if s.empty() {
                *sink = None;
                *current_file = None;
                playback.stop();
            }
        }

        let status = get_status(current_file, sink);
        let menu = build_menu(status);

        if menu.is_empty() {
            return Ok(Action::Quit);
        }
        if idx >= menu.len() {
            idx = menu.len() - 1;
        }

        clear_screen()?;
        draw_header(device_name, dir, current_file, sink, playback, volume)?;
        draw_menu(&menu, idx)?;

        if event::poll(Duration::from_millis(150))? {
            match read_key()? {
                KeyCode::Up => {
                    if idx > 0 {
                        idx -= 1;
                    }
                }
                KeyCode::Down => {
                    if idx + 1 < menu.len() {
                        idx += 1;
                    }
                }
                KeyCode::Enter => {
                    unsafe {
                        LAST_IDX = idx;
                    }
                    return Ok(menu[idx].1);
                }
                _ => {}
            }
        }
    }
}
