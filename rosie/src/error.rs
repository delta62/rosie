use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Err)), visibility(pub))]
pub enum Error {
    #[snafu(display("Error accessing {path}"))]
    Io {
        path: String,
        source: std::io::Error,
    },

    Manifest {
        source: toml::de::Error,
    },

    #[snafu(display("Unable to find an app which can handle '{command}'"))]
    UnrecognizedCommand {
        command: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
