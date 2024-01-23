use clap::Parser;

mod record_audio;

#[derive(Parser, Debug)]
#[command(version, about = "English <-> Chinese Translator", long_about = None)]
struct Opt {
    /// The audio device to use
    #[arg(short, long, default_value_t = String::from("default"))]
    device: String,
}

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();

    let wav_data = record_audio::record_audio(5)?;
    println!("{:?}", wav_data);
    record_audio::write_wav_file(&wav_data, "output.wav")?;

    Ok(())
}

