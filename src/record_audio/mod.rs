use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

pub fn record_audio(length_of_recording: u64) -> Result<Vec<u8>, anyhow::Error> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let input_config = input_device
        .default_input_config()
        .expect("Failed to get default input config");

    //our output vector
    let wav_data = Arc::new(Mutex::new(Vec::new()));
    let wav_data_clone = Arc::clone(&wav_data);

    println!("Beginning recording...");
    let input_stream = input_device
        .build_input_stream(
            &input_config.into(),
            move |data: &[f32], _: &_| {
                let mut wav_data = wav_data_clone.lock().unwrap();
                for &sample in data {
                    let byte_sample = (sample * 128.0 + 128.0) as u8;
                    wav_data.write(&[byte_sample]).expect("Failed to write to Vec");
                }
            },
            move |err| eprintln!("an error occurred on stream: {}", err),
            None,
        )
        .expect("Failed to build input stream");

    // Start the input stream
    input_stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(length_of_recording));
    drop(input_stream);
    let wav_data = wav_data.lock().unwrap().clone();
    Ok(wav_data)
}

pub fn write_wav_file(data: &[u8], file_name: &str) -> io::Result<()> {
    let mut file = File::create(file_name)?;

    // Write WAV file header
    file.write_all(b"RIFF")?;
    file.write_all(&(data.len() as u32 + 36).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?; // PCM format size
    file.write_all(&1u16.to_le_bytes())?; // PCM format code
    file.write_all(&(1u16).to_le_bytes())?; // Number of channels
    file.write_all(&(44100u32).to_le_bytes())?; // Sample rate (e.g., 44.1 kHz)
    file.write_all(&(44100u32 * 1 * 16 / 8).to_le_bytes())?; // Byte rate
    file.write_all(&(1u16 * 16 / 8).to_le_bytes())?; // Block align
    file.write_all(&(16u16).to_le_bytes())?; // Bits per sample
    file.write_all(b"data")?;
    file.write_all(&(data.len() as u32).to_le_bytes())?;

    // Write audio data
    file.write_all(data)?;
    Ok(())
}
