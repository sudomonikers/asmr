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

    let wav_data = record_audio::record_audio(4)?;
    //println!("{:?}", wav_data);
    //std::fs::write("output.wav", &wav_data)?;

    Ok(())
}

