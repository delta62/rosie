use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// Path to a ggml model to load for speech to text
    #[clap(short, long)]
    pub model: String,

    /// Path to a .wav file to process as a command
    #[clap(short, long)]
    pub wav: String,
}
