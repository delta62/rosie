use std::{fs::File, path::Path};

pub fn load_wav<P: AsRef<Path>>(path: P) -> Vec<f32> {
    let mut inp_file = File::open(path).unwrap();
    let (header, data) = wav::read(&mut inp_file).unwrap();

    debug_assert!(header.channel_count == 1, "WAV files must be mono");
    debug_assert!(
        header.sampling_rate == 16_000,
        "WAV files must be sampled at 16kHz"
    );

    data.try_into_thirty_two_float().unwrap()
}
