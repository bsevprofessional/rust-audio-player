use std::time::{Duration, Instant};

pub struct Playback {
    pub total: Option<Duration>,
    start: Option<Instant>,
    paused_at: Option<Instant>,
    paused_accum: Duration,
}

impl Playback {
    pub fn new() -> Self {
        Self {
            total: None,
            start: None,
            paused_at: None,
            paused_accum: Duration::ZERO,
        }
    }

    pub fn start_new(&mut self, total: Option<Duration>) {
        self.total = total;
        self.start = Some(Instant::now());
        self.paused_at = None;
        self.paused_accum = Duration::ZERO;
    }

    pub fn stop(&mut self) {
        self.total = None;
        self.start = None;
        self.paused_at = None;
        self.paused_accum = Duration::ZERO;
    }

    pub fn pause(&mut self) {
        if self.start.is_some() && self.paused_at.is_none() {
            self.paused_at = Some(Instant::now());
        }
    }

    pub fn resume(&mut self) {
        if let Some(p) = self.paused_at.take() {
            self.paused_accum += p.elapsed();
        }
    }

    pub fn elapsed(&self) -> Duration {
        let Some(start) = self.start else { return Duration::ZERO; };

        let raw = if let Some(p) = self.paused_at {
            p.duration_since(start).saturating_sub(self.paused_accum)
        } else {
            Instant::now().duration_since(start).saturating_sub(self.paused_accum)
        };

        if let Some(total) = self.total {
            raw.min(total)
        } else {
            raw
        }
    }

}

pub fn format_time(d: Duration) -> String {
    let secs = d.as_secs();
    let m = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", m, s)
}

pub fn progress_bar(elapsed: Duration, total: Option<Duration>, width: usize) -> String {
    let total = match total {
        Some(t) if t.as_secs_f64() > 0.0 => t,
        _ => return format!("[{}]", "-".repeat(width)),
    };

    let ratio = (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0);
    let filled = ((ratio * width as f64).round() as usize).min(width);

    let left = "#".repeat(filled);
    let right = "-".repeat(width - filled);
    format!("[{}{}]", left, right)
}

pub fn indeterminate_bar(elapsed: Duration, width: usize) -> String {
    if width == 0 {
        return "[]".to_string();
    }

    let t = elapsed.as_millis() as usize;
    let pos = (t / 120) % width; // speed

    let mut s = String::with_capacity(width + 2);
    s.push('[');
    for i in 0..width {
        s.push(if i == pos { '>' } else { '-' });
    }
    s.push(']');
    s
}

pub fn percent(elapsed: Duration, total: Option<Duration>) -> Option<f64> {
    let total = total?;
    if total.as_secs_f64() <= 0.0 {
        return None;
    }
    Some((elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0))
}
