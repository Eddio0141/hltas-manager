use super::games::game_dir_types;
use crate::{
    cfg::{self, Cfg},
    helper,
};
use anyhow::{bail, Context, Result};
use log::{debug, info};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn sync_saves(keep_alive: bool) -> Result<()> {
    let root_dir = helper::try_root_dir()
        .context("Failed to get root dir")?
        .path;
    // load config
    let config_path = root_dir.join(cfg::cfg_file_name());
    let config = Cfg::load(config_path).context("Failed to load config")?;

    // paths
    let save = Path::new("SAVE");
    let half_life_dir = root_dir.join(&config.half_life_dir);
    let half_life_second_dir = match &config.no_client_dll_dir {
        Some(dir) => root_dir.join(dir),
        None => bail!(
            "No no-client-dll dir set, can't sync saves\nHelp: Install using the command 'install'"
        ),
    };

    let keep_alive_interval = Duration::from_secs(1);

    if keep_alive {
        loop {
            sync_saves_once(&save, &half_life_dir, &half_life_second_dir)?;
            std::thread::sleep(keep_alive_interval);
        }
    } else {
        sync_saves_once(save, half_life_dir, half_life_second_dir)?;
        info!("Synced saves!");
    }

    Ok(())
}

fn sync_saves_once<S: AsRef<Path>, P: AsRef<Path>, P2: AsRef<Path>>(
    saves_dir: S,
    half_life_dir: P,
    half_life_second_dir: P2,
) -> Result<()> {
    let half_life_dir = half_life_dir.as_ref();
    let half_life_second_dir = half_life_second_dir.as_ref();

    // we only do the copies for the games in second game dir
    let games =
        game_dir_types(&half_life_second_dir).context("Failed to get games from second dir")?;

    for game in games {
        let first_dir_saves_dir = half_life_dir.join(&game.name).join(&saves_dir);
        let second_dir_saves_dir = half_life_second_dir.join(&game.name).join(&saves_dir);

        // create all dir for missing saves folder
        if !first_dir_saves_dir.is_dir() {
            info!(
                "Creating saves dir for {} in first half-life dir",
                game.name
            );
            fs::create_dir_all(&first_dir_saves_dir).context("Failed to create saves dir")?;
        }
        if !second_dir_saves_dir.is_dir() {
            info!(
                "Creating saves dir for {} in second half-life dir",
                game.name
            );
            fs::create_dir_all(&second_dir_saves_dir).context("Failed to create saves dir")?;
        }

        let saves_from_dir = |dir: &Path| -> Result<Vec<PathBuf>> {
            let mut saves = Vec::new();

            for dir in dir.read_dir().context("Failed to read dir")? {
                let dir = dir.context("Failed to read dir")?;
                let path = dir.path();

                if let Some(extension) = path.extension() {
                    if extension == "sav" {
                        saves.push(path);
                    }
                }
            }

            Ok(saves)
        };

        // get list of save files from each dir
        let first_dir_saves = saves_from_dir(&first_dir_saves_dir)?;
        let mut second_dir_saves = saves_from_dir(&second_dir_saves_dir)?;

        for path in first_dir_saves {
            // check if file exists in second dir
            let src_dest = match second_dir_saves
                .iter()
                .enumerate()
                .find(|(_, p)| p.file_name() == path.file_name())
            {
                Some((dupe_i, dupe)) => {
                    // if it does, check which is modified or not
                    let first_save_modified = path
                        .metadata()
                        .context("Failed to get metadata")?
                        .modified()
                        .context("Failed to get modified time")?;

                    let dupe_modified = dupe
                        .metadata()
                        .context("Failed to get metadata")?
                        .modified()
                        .context("Failed to get modified time")?;

                    debug!(
                        "first save ({}) modified date: {:?}, dupe ({}) modified date: {:?}",
                        path.display(),
                        first_save_modified,
                        dupe.display(),
                        dupe_modified
                    );

                    let src_dest = match first_save_modified.cmp(&dupe_modified) {
                        std::cmp::Ordering::Less => Some((dupe.clone(), path)),
                        std::cmp::Ordering::Equal => None,
                        std::cmp::Ordering::Greater => Some((path, dupe.clone())),
                    };

                    // we don't want to copy the same file twice
                    second_dir_saves.remove(dupe_i);

                    src_dest
                }
                None => {
                    // if it doesn't, copy it
                    Some((
                        path.clone(),
                        second_dir_saves_dir
                            .join(path.file_name().context("Failed to get file name")?),
                    ))
                }
            };

            // overwrite copy in second dir
            if let Some((src, dest)) = src_dest {
                info!("Copying from {} to {}", src.display(), dest.display());
                fs::copy(src, dest).context("Failed to copy save file")?;
            }
        }

        // copy remaining files from second dir to first dir
        for path in second_dir_saves {
            let dest =
                first_dir_saves_dir.join(path.file_name().context("Failed to get file name")?);

            info!("Copying from {} to {}", path.display(), dest.display());
            fs::copy(&path, dest).context("Failed to copy save file")?;
        }
    }

    Ok(())
}
