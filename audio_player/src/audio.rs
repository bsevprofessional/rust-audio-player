use anyhow::{Context, Result};
use rodio::{Decoder, Sink};
use std::{fs::File, io::BufReader, path::Path, time::Duration};

use crate::playback::Playback;

pub fn play_file(
    path: &Path,
    handle: &rodio::OutputStreamHandle,
    sink: &mut Option<Sink>,
    playback: &mut Playback,
) -> Result<()> {
    if let Some(s) = sink.take() {
        s.stop();
    }

    let total = probe_duration(path);

    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let reader = BufReader::new(file);
    let decoded = Decoder::new(reader).context("Failed to decode audio")?;

    let new_sink = Sink::try_new(handle).context("Failed to create sink")?;
    new_sink.append(decoded);
    new_sink.play();

    *sink = Some(new_sink);
    playback.start_new(total);
    Ok(())
}

pub fn probe_duration(path: &Path) -> Option<Duration> {
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = std::fs::File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let mut format = probed.format;

    let track_id = format.default_track().map(|t| t.id)?;

    let track = format.tracks().iter().find(|t| t.id == track_id)?;
    let params = &track.codec_params;

    let sample_rate = params.sample_rate? as u64;

    if let Some(n_frames) = params.n_frames {
        let secs = n_frames as f64 / sample_rate as f64;
        return Some(Duration::from_secs_f64(secs));
    }

    // Conservative fallback: estimate by decoding frames (bounded)
    let mut decoder = symphonia::default::get_codecs()
        .make(params, &DecoderOptions::default())
        .ok()?;

    let mut total_samples: u64 = 0;
    let mut packets_read: u32 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        if let Ok(audio_buf) = decoder.decode(&packet) {
            total_samples += audio_buf.frames() as u64;
        }

        packets_read += 1;
        if packets_read > 5000 {
            break;
        }
    }

    if total_samples == 0 {
        None
    } else {
        Some(Duration::from_secs_f64(total_samples as f64 / sample_rate as f64))
    }
}