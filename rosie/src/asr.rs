use crate::wav::load_wav;
use std::path::Path;
use whisper::{Params, SamplingStrategy, WhisperContext};

pub fn parse_sample<P1, P2>(model_path: P1, wav_path: P2) -> String
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let bytes = load_wav(wav_path);
    let mut ctx = WhisperContext::from_file(model_path.as_ref()); //"data/ggml-base.en.bin");
    let params = Params::new(SamplingStrategy::Greedy);

    ctx.run_full(params, &bytes)
}
