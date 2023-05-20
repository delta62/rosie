use crate::error::Error;
use crate::error::{IoErr, ManifestErr, Result, UnrecognizedCommandErr};
use futures::TryStreamExt;
use rlua::{Function, Lua, Table};
use serde::Deserialize;
use snafu::{OptionExt, ResultExt};
use std::{collections::HashMap, path::Path};
use tokio::fs::{read_dir, read_to_string, DirEntry};
use tokio_stream::wrappers::ReadDirStream;

#[derive(Debug, Deserialize)]
struct Manifest {
    id: String,
    trigger_phrase: String,
}

pub struct AppHost {
    apps: HashMap<String, Manifest>,
}

impl AppHost {
    pub async fn from_dir<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let entries = read_dir(path).await.context(IoErr {
            path: path.to_str().unwrap().to_owned(),
        })?;

        let apps = ReadDirStream::new(entries)
            .map_err(|source| Error::Io {
                path: path.to_string_lossy().into_owned(),
                source,
            })
            .and_then(|entry| load_manifest(entry))
            .map_ok(|manifest| (manifest.id.clone(), manifest))
            .try_collect()
            .await?;

        Ok(AppHost { apps })
    }

    pub async fn dispatch(&self, command: &str) -> Result<()> {
        let app_id = self
            .apps
            .iter()
            .find(|(_, app)| command.starts_with(&app.trigger_phrase))
            .context(UnrecognizedCommandErr {
                command: command.to_owned(),
            })?
            .0;

        log::debug!("Found app {app_id} which can handle this command");
        self.invoke_app(app_id, command).await?;

        Ok(())
    }

    async fn invoke_app(&self, app_id: &str, command: &str) -> Result<()> {
        let app_path = Path::new("apps").join(app_id);
        let main = app_path.join("main.lua");
        let main = read_to_string(&main).await.context(IoErr {
            path: app_path.as_path().to_string_lossy().to_owned(),
        })?;

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
            args.set("phrase", command)?;

            main.call::<Table, ()>(args)?;

            Ok::<(), rlua::Error>(())
        })
        .unwrap();

        Ok(())
    }
}

async fn load_manifest(entry: DirEntry) -> Result<Manifest> {
    let path = entry.path();
    let path = path.as_path();
    let manifest_path = path.join("manifest.toml");
    let manifest = read_to_string(&manifest_path).await.context(IoErr {
        path: manifest_path.to_string_lossy().to_owned(),
    })?;

    toml::from_str::<Manifest>(&manifest).context(ManifestErr)
}
