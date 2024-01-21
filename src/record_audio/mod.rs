use std::sync::{Arc, Mutex};
use std::io::BufWriter;
use std::fs::File;
use cpal::{FromSample, Sample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::WavWriter;
use std::io::Cursor;

pub fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

pub fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;
pub fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
    where
        T: Sample,
        U: Sample + hound::Sample + FromSample<T>,
    {
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

pub fn record_audio(length_of_recording: u64) -> Result<Vec<u8>, anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("failed to find input device");
    let input_config = input_device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", input_config);

    // The WAV file we're recording to.
    let spec = wav_spec_from_config(&input_config);
    let mut writer = WavWriter::new(Cursor::new(Vec::new()), spec)?;


    println!("Begin recording...");
    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    let stream = match input_config.sample_format() {
        cpal::SampleFormat::I8 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
            move |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?,
        cpal::SampleFormat::I16 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
            move |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?,
        cpal::SampleFormat::I32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
            move |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?,
        cpal::SampleFormat::F32 => input_device.build_input_stream(
            &input_config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
            move |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )?,
        sample_format => {
            return Err(anyhow::Error::msg(format!(
                "Unsupported sample format '{sample_format}'"
            )))
        }
    };

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(length_of_recording));
    drop(stream);

    let cursor = writer.finalize()?;
    let wav_data = cursor.into_inner();
    Ok(wav_data)
}