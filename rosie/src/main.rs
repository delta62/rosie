mod wav;

use whisper::{Params, SamplingStrategy, WhisperContext};

use crate::wav::load_wav;

fn main() {
    let sample_file = std::env::args().nth(1).unwrap();
    let bytes = load_wav(sample_file);

    let mut ctx = WhisperContext::from_file("data/ggml-base.en.bin");
    let params = Params::new(SamplingStrategy::Greedy);
    let text = ctx.run_full(params, &bytes);

    println!("{text}");
}
