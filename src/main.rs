mod error;
mod synth;
mod synth_ui;
/// TODO: Callback, Github, Panning, Filter
use error::{BaseError, Result};
use synth::{SampleFormat, Synth};

use druid::{AppLauncher, WindowDesc};
use std::sync::{mpsc, Arc, Mutex};
use synth_ui::{build_ui, SynthUIData, SynthUIEvent};

use portaudio_rs as pa;

const SAMPLE_RATE: f32 = 44100.0;
const CHANNELS_NUM: usize = 2;
const BUF_SIZE: u32 = 600;

fn create_output_stream<SF>(
    sample_rate: f32,
    buf_size: u32,
    channels_num: u32,
    callback: Option<Box<pa::stream::StreamCallback<'static, SF, SF>>>
) -> Result<pa::stream::Stream<'_, SF, SF>>
where
    SF: SampleFormat,
{
    let default_output = match pa::device::get_default_output_index() {
        Some(dev) => dev,
        None => {
            return Err(BaseError::StreamError(
                "Can't open default device".into(),
            ))
        }
    };

    let latency = match pa::device::get_info(default_output) {
        Some(info) => info.default_low_output_latency,
        None => return Err(BaseError::StreamError("Can't get latency info".to_owned())),
    };

    let output_params = pa::stream::StreamParameters::<SF> {
        device: default_output,
        channel_count: channels_num,
        suggested_latency: latency,
        data: SF::min_value(),
    };

    let _supported =
        pa::stream::is_format_supported::<SF, SF>(None, Some(output_params), sample_rate as f64)?;

    let stream = match pa::stream::Stream::<SF, SF>::open(
        None,
        Some(output_params),
        sample_rate as f64,
        buf_size as u64,
        pa::stream::StreamFlags::empty(),
        callback,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening stream: {}", e);
            return Err(BaseError::PaError(e));
        }
    };

    Ok(stream)
}

fn main() -> Result<()> {
    let mut synth = Synth::<i16>::new(SAMPLE_RATE);
    synth.set_volume(-36)?;
    let synth_arc = Arc::new(Mutex::new(synth));

    let (synth_event, wait_synth_event): (mpsc::Sender<SynthUIEvent>, mpsc::Receiver<SynthUIEvent>) = mpsc::channel();

    let synth_in_thread = Arc::clone(&synth_arc);
    let th = std::thread::Builder::new()
        .name("beep-boop-synth".into())
        .spawn(move || -> Result<()> {
            pa::initialize()?;
            let synth = synth_in_thread;
            let synth_callback = Arc::clone(&synth);
            let (stream_finished, wait_stream_finished): (mpsc::Sender<()>, mpsc::Receiver<()>) = mpsc::channel();
            let callback = Box::new(
                move |
                    _input: &[i16],
                    output: &mut [i16],
                    _time: pa::stream::StreamTimeInfo,
                    _flags: pa::stream::StreamCallbackFlags| -> pa::stream::StreamCallbackResult
                    {
                        let mut synth = synth_callback.lock().unwrap();
                        if !synth.playing() {
                            stream_finished.send(()).unwrap();
                            return pa::stream::StreamCallbackResult::Complete
                        }
                        let mut sample = synth.next().unwrap();
                        let mut written = 0;
                        for i in 0..output.len() {
                            output[i] = sample;
                            written += 1;
                            if written == CHANNELS_NUM {
                                sample = synth.next().unwrap();
                                written = 0;
                            }
                        }
                        pa::stream::StreamCallbackResult::Continue
                    }
            );
            let stream = create_output_stream::<i16>(SAMPLE_RATE, BUF_SIZE, CHANNELS_NUM as u32, Some(callback))?;

            'synthloop: loop {
                match wait_synth_event.recv() {
                    Ok(SynthUIEvent::NewNotes) => {
                        if !stream.is_active()? {
                            stream.start()?
                        }
                        wait_stream_finished.recv().unwrap();
                        if stream.is_active()? {
                            stream.stop()?
                        }
                    },
                    Ok(SynthUIEvent::WindowClosed) | Err(_) => {
                        break 'synthloop
                    },
                }
            }
            drop(stream);
            pa::terminate()?;
            Ok(())
        });

    let _th = match th {
        Ok(handler) => handler,
        Err(_) => return Err(BaseError::ThreadError("Can't start synth thread".into())),
    };

    {
        let window = WindowDesc::new(build_ui)
            .title("beep-boop")
            .with_min_size((860.0, 550.0))
            .resizable(false);
        let launcher = AppLauncher::with_window(window);

        launcher
            .delegate(synth_ui::Delegate)
            .launch(SynthUIData::new(synth_arc, synth_event, SAMPLE_RATE))
            .expect("Starting beep-boop GUI failed :(");
    }

    _th.join().expect("Synth thread join error")?;
    Ok(())
}
