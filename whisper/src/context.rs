use std::{
    ffi::{CStr, CString},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};
use whisper_rs_sys::{
    whisper_context, whisper_free, whisper_full, whisper_full_default_params,
    whisper_full_get_segment_text, whisper_full_n_segments, whisper_full_params,
    whisper_init_from_file, whisper_sampling_strategy_WHISPER_SAMPLING_BEAM_SEARCH,
    whisper_sampling_strategy_WHISPER_SAMPLING_GREEDY,
};

pub struct WhisperContext {
    ctx: *mut whisper_context,
}

pub struct Params {
    params: whisper_full_params,
}

#[derive(Debug, Copy, Clone)]
pub enum SamplingStrategy {
    /// similar to OpenAI's GreedyDecoder
    Greedy,
    /// similar to OpenAI's BeamSearchDecoder
    BeamSearch,
}

impl Params {
    pub fn new(strategy: SamplingStrategy) -> Self {
        let c_strat = match strategy {
            SamplingStrategy::Greedy => whisper_sampling_strategy_WHISPER_SAMPLING_GREEDY,
            SamplingStrategy::BeamSearch => whisper_sampling_strategy_WHISPER_SAMPLING_BEAM_SEARCH,
        };

        let params = unsafe { whisper_full_default_params(c_strat) };
        Self { params }
    }
}

impl WhisperContext {
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        let pathbuf: PathBuf = path.into();
        let path = pathbuf.as_os_str().as_bytes();
        let cstr = CString::new(path).unwrap();
        let ctx = unsafe { whisper_init_from_file(cstr.as_ptr()) };

        Self { ctx }
    }

    pub fn run_full(&mut self, params: Params, samples: &[f32]) -> String {
        let n_samples = samples.len().try_into().unwrap();
        let n_segments;
        unsafe {
            whisper_full(self.ctx, params.params, samples.as_ptr(), n_samples);
            n_segments = whisper_full_n_segments(self.ctx);
        }

        let mut ret = String::new();
        for i in 0..n_segments {
            let text = unsafe { whisper_full_get_segment_text(self.ctx, i) };
            let s = unsafe { CStr::from_ptr(text) };
            ret.push(' ');
            ret.push_str(s.to_str().unwrap());
        }

        ret
    }
}

impl Drop for WhisperContext {
    fn drop(&mut self) {
        unsafe { whisper_free(self.ctx) }
    }
}
