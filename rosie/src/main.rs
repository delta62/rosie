use std::fs::File;
use std::path::Path;

use whisper::{Params, SamplingStrategy, WhisperContext};

fn main() {
    let sample_file = std::env::args().nth(1).unwrap();

    let mut inp_file = File::open(Path::new(&sample_file)).unwrap();
    let (_, data) = wav::read(&mut inp_file).unwrap();
    let bytes = data.try_into_thirty_two_float().unwrap();

    let mut ctx = WhisperContext::from_file("data/ggml-base.en.bin");
    let params = Params::new(SamplingStrategy::Greedy);
    let text = ctx.run_full(params, &bytes);

    println!("{text}");
}
