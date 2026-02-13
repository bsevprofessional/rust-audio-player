use rodio::Sink;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub enum Status {
    NoFile,
    Playing,
    Paused,
}

pub fn get_status(current_file: &Option<PathBuf>, sink: &Option<Sink>) -> Status {
    if current_file.is_none() || sink.is_none() {
        return Status::NoFile;
    }
    let s = sink.as_ref().unwrap();
    if s.is_paused() {
        Status::Paused
    } else {
        Status::Playing
    }
}

#[derive(Clone, Copy)]
pub enum Action {
    SelectFile,
    PauseResume,
    Stop,
    VolumeUp,
    VolumeDown,
    Quit,
}

pub fn build_menu(status: Status) -> Vec<(String, Action)> {
    // Volume should always be available (even with no file selected)
    let mut base = vec![
        ("Volume +".to_string(), Action::VolumeUp),
        ("Volume -".to_string(), Action::VolumeDown),
    ];

    match status {
        Status::NoFile => {
            let mut m = vec![("Select file and play".to_string(), Action::SelectFile)];
            m.append(&mut base);
            m.push(("Quit".to_string(), Action::Quit));
            m
        }
        Status::Playing => {
            let mut m = vec![
                ("Select file and play".to_string(), Action::SelectFile),
                ("Pause".to_string(), Action::PauseResume),
                ("Stop".to_string(), Action::Stop),
            ];
            m.append(&mut base);
            m.push(("Quit".to_string(), Action::Quit));
            m
        }
        Status::Paused => {
            let mut m = vec![
                ("Select file and play".to_string(), Action::SelectFile),
                ("Resume".to_string(), Action::PauseResume),
                ("Stop".to_string(), Action::Stop),
            ];
            m.append(&mut base);
            m.push(("Quit".to_string(), Action::Quit));
            m
        }
    }
}
