use crate::error::Error;
use crate::error::{IoErr, ManifestErr, Result, UnrecognizedCommandErr};
use futures::TryStreamExt;
use rlua::{Function, Lua, Table};
use serde::Deserialize;
use snafu::{OptionExt, ResultExt};
use std::{collections::HashMap, path::Path};
use tokio::{
    fs::{read_dir, read_to_string, DirEntry},
    time::Duration,
};
use tokio_stream::wrappers::ReadDirStream;

#[derive(Debug, Deserialize)]
struct Manifest {
    id: String,
    trigger_phrase: String,
}

#[derive(Debug)]
pub struct Delayed {
    app_id: String,
    duration: Duration,
}

pub struct AppHost<S>
where
    S: Fn(Delayed) + Send + Sync,
{
    apps: HashMap<String, Manifest>,
    sender: S,
}

impl<S> AppHost<S>
where
    S: Fn(Delayed) + Send + Sync,
{
    pub async fn from_dir<P: AsRef<Path>>(path: P, sender: S) -> Result<Self> {
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

        Ok(AppHost { apps, sender })
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

        log::debug!("dispatching to app_id '{app_id}'");
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
            let result = ctx.scope(|scope| {
                let api = ctx.create_table()?;
                api.set(
                    "delayed",
                    scope.create_function(|_ctx, millis: u64| {
                        println!("Schedule delayed task");
                        let delayed = Delayed {
                            app_id: app_id.to_owned(),
                            duration: Duration::from_millis(millis),
                        };
                        (self.sender)(delayed);
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
            });

            if let Err(e) = result {
                log::error!("Oh noes: {e}");
            }

            Ok(())
        })
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
