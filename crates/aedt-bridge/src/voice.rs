//! Voice input: the reader's words, recorded from the microphone and written
//! down by the provider's speech-to-text.

use std::{
    cell::RefCell,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::{self, TryRecvError},
    },
    thread,
    time::Duration,
};

use aedt_provider::speech;
use aedt_settings as settings;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};
use nsh_core::prelude::*;

mod shortcut;

/// The rate a recording is written at: what a provider reads best, and small
/// enough that the longest one stays well under a request's limit.
const RATE: u32 = 16_000;

/// How long a recording may run: a speech-to-text request takes no more.
const MAX_SECONDS: u32 = 120;

/// What the voice button shows, and what the composer takes from it.
#[derive(Clone, Default)]
pub struct Voice {
    /// The microphone is on.
    pub recording: bool,
    /// The recording is with the provider, waiting for its words.
    pub transcribing: bool,
    /// What the microphone heard, for the composer's draft to take.
    pub heard: Option<SharedString>,
    /// Why the last attempt failed, for the composer to show.
    pub error: Option<SharedString>,
}

/// The voice button's state, one for the app (TS: a signal at module level).
pub fn state() -> Signal<Voice> {
    global::<Voice>()
}

/// The microphone's loudness, for the bars the composer floats while it
/// listens.
#[derive(Clone, Copy, Default)]
pub struct Meter {
    /// How loud the last moment was: full scale at 1.0.
    pub level: f32,
    /// How many measurements have come: the bars ripple along the row as they
    /// arrive.
    pub frame: u32,
}

/// The microphone's loudness, one for the app.
pub fn meter() -> Signal<Meter> {
    global::<Meter>()
}

/// How far the three dots have come, while the provider writes a recording
/// down.
#[derive(Clone, Copy, Default)]
pub struct Dots(pub u32);

/// The three dots the composer turns while it waits, one for the app.
pub fn dots() -> Signal<Dots> {
    global::<Dots>()
}

thread_local! {
    /// The recording in flight, if any. The engine thread keeps the handle; the
    /// capture thread reads the stop flag and fills the samples.
    static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

/// One recording: the flag that ends it, and the thread that reads the
/// microphone.
struct Recorder {
    stop: Arc<AtomicBool>,
    thread: thread::JoinHandle<()>,
}

/// Starts recording. `Err` says why the microphone thread could not start; a
/// microphone that cannot open reports itself through the state instead.
pub fn start() -> Result<(), String> {
    if state().get_untracked().recording {
        return Ok(());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::new()));
    let level = Arc::new(AtomicU32::new(0));
    let (sender, recorded) = mpsc::channel::<Result<Option<Vec<u8>>, String>>();
    let thread = {
        let stop = Arc::clone(&stop);
        let samples = Arc::clone(&samples);
        let level = Arc::clone(&level);
        thread::Builder::new()
            .name("aedt-voice".to_owned())
            .spawn(move || capture(&stop, &samples, &level, &sender))
            .map_err(|error| format!("the microphone thread did not start: {error}"))?
    };
    RECORDER.with(|slot| *slot.borrow_mut() = Some(Recorder { stop, thread }));
    state().update(|voice| {
        voice.recording = true;
        voice.error = None;
    });

    // Under no owner of its own: a pane going away must not stop the words
    // from coming back into whatever composer shows next.
    Owner::detached().run(|| spawn(async move { hear(recorded, level).await }));

    Ok(())
}

/// Asks the recording to end: what it heard goes to the provider.
pub fn stop() {
    if !state().get_untracked().recording {
        return;
    }
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow().as_ref() {
            recorder.stop.store(true, Ordering::Relaxed);
        }
    });
    state().update(|voice| {
        voice.recording = false;
        voice.transcribing = true;
    });
}

/// Starts a recording, or ends the one in flight: what the composer's
/// microphone and the reader's own key both do.
pub fn toggle() {
    let voice = state().get_untracked();

    if voice.recording {
        stop();

        return;
    }
    if voice.transcribing || settings::speech().is_none() {
        return;
    }

    if let Err(why) = start() {
        state().update(|voice| voice.error = Some(why.into()));
    }
}

/// Has the reader's own keys start and end a recording: the right Option on a
/// Mac toggles it, and Escape ends it. Once for the app.
pub fn watch() {
    shortcut::watch();
}

/// Takes what the composer consumed: the words it wrote down, and the failure
/// it showed.
pub fn take() {
    state().update(|voice| {
        voice.heard = None;
        voice.error = None;
    });
}

/// Waits for the recording to end, then has the provider write it down. The
/// capture thread reports the file it made, why it made none, or that it heard
/// nothing at all.
async fn hear(recorded: mpsc::Receiver<Result<Option<Vec<u8>>, String>>, level: Arc<AtomicU32>) {
    let meter = meter();
    let mut frame = 0u32;
    let mut tick = 0u8;
    let outcome = loop {
        match recorded.try_recv() {
            Ok(outcome) => break outcome,
            Err(TryRecvError::Empty) => {
                // Every other turn, near thirty a second: the bars follow
                // the voice rather than step.
                if tick == 0 {
                    frame = frame.wrapping_add(1);
                    meter.set(Meter {
                        level: f32::from_bits(level.load(Ordering::Relaxed)),
                        frame,
                    });
                }
                tick = (tick + 1) % 2;
                timer(Duration::from_millis(16)).await;
            }
            Err(TryRecvError::Disconnected) => break Err("the microphone went away".to_owned()),
        }
    };
    meter.set(Meter::default());
    // The thread is done once it has sent: joining it reaps it without waiting.
    RECORDER.with(|slot| {
        if let Some(recorder) = slot.borrow_mut().take() {
            let _ = recorder.thread.join();
        }
    });
    state().update(|voice| voice.recording = false);

    let wav = match outcome {
        Ok(Some(wav)) => wav,
        // The microphone heard nothing worth sending: no request, and nothing
        // to tell the reader.
        Ok(None) => return settle(),
        Err(why) => return fail(why),
    };
    let Some(provider) = settings::speech() else {
        return fail("no DeepSeek provider is set up to write speech down".to_owned());
    };

    state().update(|voice| voice.transcribing = true);
    turn_dots();
    let heard = speech::transcribe(&provider.base_url, &provider.api_key, wav).await;
    match heard {
        Ok(text) if text.is_empty() => settle(),
        Ok(text) => state().update(|voice| {
            voice.transcribing = false;
            voice.heard = Some(text.into());
        }),
        Err(why) => fail(why),
    }
}

/// Ends the attempt with nothing to say: the reader said nothing, and a prompt
/// for silence is noise.
fn settle() {
    state().update(|voice| {
        voice.recording = false;
        voice.transcribing = false;
        voice.heard = None;
        voice.error = None;
    });
}

/// Turns the three dots the veil shows while the provider writes the recording
/// down, until nothing is being written any more.
fn turn_dots() {
    let dots = dots();

    Owner::detached().run(|| {
        spawn(async move {
            loop {
                if !state().get_untracked().transcribing {
                    break;
                }
                dots.set(Dots(dots.get_untracked().0.wrapping_add(1)));
                timer(Duration::from_millis(32)).await;
            }
        })
    });
}

/// Ends the attempt with `why`, for the composer to show.
fn fail(why: impl Into<SharedString>) {
    state().update(|voice| {
        voice.recording = false;
        voice.transcribing = false;
        voice.error = Some(why.into());
    });
}

/// Records until `stop` is set or the longest recording is reached, then sends
/// the file it made, nothing when the microphone heard nothing, or why it made
/// none.
fn capture(
    stop: &AtomicBool,
    samples: &Arc<Mutex<Vec<f32>>>,
    level: &Arc<AtomicU32>,
    done: &mpsc::Sender<Result<Option<Vec<u8>>, String>>,
) {
    let result = listen(stop, samples, level).map(|rate| {
        let mono = samples
            .lock()
            .map(|samples| samples.clone())
            .unwrap_or_default();
        if mono.is_empty() {
            return None;
        }

        let pcm = resample(&mono, rate, RATE);

        Some(wav(&pcm, rate.min(RATE)))
    });

    let _ = done.send(result);
}

/// Opens the default microphone and fills `samples` with it at its own rate
/// until `stop` is set or the longest recording is reached. `Err` says why the
/// microphone could not be used.
fn listen(
    stop: &AtomicBool,
    samples: &Arc<Mutex<Vec<f32>>>,
    level: &Arc<AtomicU32>,
) -> Result<u32, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "no microphone is connected".to_owned())?;
    let supported = device
        .default_input_config()
        .map_err(|error| format!("the microphone could not be opened: {error}"))?;
    let format = supported.sample_format();
    let rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    let config: StreamConfig = supported.into();
    let limit = rate as usize * MAX_SECONDS as usize;

    let stream = match format {
        SampleFormat::F32 => input::<f32>(&device, config, channels, samples, level, limit),
        SampleFormat::I16 => input::<i16>(&device, config, channels, samples, level, limit),
        SampleFormat::U16 => input::<u16>(&device, config, channels, samples, level, limit),
        SampleFormat::I32 => input::<i32>(&device, config, channels, samples, level, limit),
        SampleFormat::F64 => input::<f64>(&device, config, channels, samples, level, limit),
        other => {
            return Err(format!(
                "the microphone's samples are {other}, which this app does not read"
            ));
        }
    }
    .map_err(|error| format!("the microphone could not be opened: {error}"))?;
    stream
        .play()
        .map_err(|error| format!("the microphone did not start: {error}"))?;

    // The stream records on the driver's own thread; this one waits for the
    // reader to stop it, or for the longest recording to fill.
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let full = samples.lock().is_ok_and(|samples| samples.len() >= limit);
        if full {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    drop(stream);

    Ok(rate)
}

/// The microphone `device` as a stream of mono samples in `samples`, at most
/// `limit` of them. Stereo and more is cut to its first channel, which is the
/// one a provider reads.
fn input<T>(
    device: &cpal::Device,
    config: StreamConfig,
    channels: usize,
    samples: &Arc<Mutex<Vec<f32>>>,
    level: &Arc<AtomicU32>,
    limit: usize,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let samples = Arc::clone(samples);
    let level = Arc::clone(level);

    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let Ok(mut samples) = samples.lock() else {
                return;
            };
            let mut squares = 0.0;
            let mut count = 0usize;
            for frame in data.chunks(channels) {
                if samples.len() >= limit {
                    break;
                }
                let sample = f32::from_sample_(frame[0]);
                squares += sample * sample;
                count += 1;
                samples.push(sample);
            }
            if count > 0 {
                let rms = (squares / count as f32).sqrt();
                level.store(loudness(rms).to_bits(), Ordering::Relaxed);
            }
        },
        |error| eprintln!("voice: {error}"),
        None,
    )
}

/// `rms` (full scale at 1.0) as the meter's 0..1: speech sits near a tenth of
/// full scale, so the middle of the meter is ordinary talk.
fn loudness(rms: f32) -> f32 {
    (rms * 3.0).clamp(0.0, 1.0).sqrt()
}

/// `samples` at `from` Hz taken down to `to` Hz: each one the mean of the
/// input it covers. A recording no faster than `to` is left as it is.
fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from <= to || samples.is_empty() {
        return samples.to_vec();
    }

    let step = f64::from(from) / f64::from(to);
    let count = (samples.len() as f64 / step).floor() as usize;

    let mut out = Vec::with_capacity(count);
    for index in 0..count {
        let start = (index as f64 * step).floor() as usize;
        let end = (((index + 1) as f64 * step).floor() as usize)
            .max(start + 1)
            .min(samples.len());
        let window = &samples[start..end];
        out.push(window.iter().sum::<f32>() / window.len() as f32);
    }

    out
}

/// `samples` as a 16-bit PCM WAV file at `rate` Hz: the one file a
/// speech-to-text request carries.
fn wav(samples: &[f32], rate: u32) -> Vec<u8> {
    let data = samples.len() * 2;

    let mut out = Vec::with_capacity(44 + data);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data as u32).to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        out.extend_from_slice(&value.to_le_bytes());
    }

    out
}
