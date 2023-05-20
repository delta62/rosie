mod wav;

use std::fs::{read_dir, read_to_string};
use std::path::Path;

use crate::wav::load_wav;
use rlua::{Function, Lua, Table};
use serde::Deserialize;
use whisper::{Params, SamplingStrategy, WhisperContext};

fn parse_sample() {
    let sample_file = std::env::args().nth(1).unwrap();
    let bytes = load_wav(sample_file);

    let mut ctx = WhisperContext::from_file("data/ggml-base.en.bin");
    let params = Params::new(SamplingStrategy::Greedy);
    let text = ctx.run_full(params, &bytes);

    println!("{text}");
}

#[derive(Debug, Deserialize)]
struct Manifest {
    id: String,
    name: String,
    trigger_phrase: String,
}

fn run_app<P: AsRef<Path>>(manifest: &Manifest, app_path: P) {
    println!("Running app {} [{}]", manifest.name, manifest.id);
    println!(
        "This app responds to the trigger phrase '{}'",
        manifest.trigger_phrase
    );

    let main = app_path.as_ref().join("main.lua");
    let main = read_to_string(&main).unwrap();

    let ctx = Lua::new();

    ctx.context(|ctx| {
        let api = ctx.create_table()?;

        api.set(
            "delayed",
            ctx.create_function(|_ctx, ()| {
                println!("Schedule delayed task");
                Ok(())
            })?,
        )?;
        let chunk = ctx.load(&main);
        chunk.exec().unwrap();

        let globals = ctx.globals();
        globals.set("api", api)?;
        let main: Function = globals.get("main")?;

        let args = ctx.create_table()?;
        args.set("phrase", "blahblah")?;

        main.call::<Table, ()>(args)?;

        Ok::<(), rlua::Error>(())
    })
    .unwrap();
}

fn main() {
    let entries = read_dir("apps").unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let app_path = entry.path();

        let manifest_path = app_path.join("manifest.toml");
        let manifest = read_to_string(&manifest_path).unwrap();
        let manifest = toml::from_str(&manifest).unwrap();

        run_app(&manifest, &app_path);
    }
}
