mod apps;
mod args;
mod asr;
mod error;
mod wav;

use apps::{AppHost, Delayed};
use args::Args;
use clap::Parser;
use error::Result;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let run_result = try_run(&args).await;

    if let Err(err) = run_result {
        log::error!("{:#?}", err);
        std::process::exit(1);
    }
}

async fn try_run(args: &Args) -> Result<()> {
    log::debug!("{:#?}", args);
    let apps = AppHost::from_dir("apps", on_message).await?;
    let command = asr::parse_sample(&args.model, &args.wav);
    log::info!("Parsed voice command: '{command}'");

    apps.dispatch(&command).await?;

    Ok(())
}

fn on_message(delayed: Delayed) {
    log::info!("I got a message! It is: {:?}.", delayed);
}
