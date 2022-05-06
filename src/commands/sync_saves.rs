use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};

use crate::{
    cfg::{self, Cfg},
    helper,
};

use super::games::game_dir_types;

pub fn sync_saves() -> Result<()> {
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
        Some(dir) => dir,
        None => bail!(
            "No no-client-dll dir set, can't sync saves\nHelp: Install using the command 'install'"
        ),
    };

    // we only do the copies for the games in second game dir
    let games =
        game_dir_types(&half_life_second_dir).context("Failed to get games from second dir")?;

    for game in games {
        let first_dir_saves_dir = half_life_dir.join(&game.name).join(save);
        let second_dir_saves_dir = half_life_second_dir.join(&game.name).join(save);

        // create all dir for missing saves folder
        if !first_dir_saves_dir.is_dir() {
            fs::create_dir_all(&first_dir_saves_dir).context("Failed to create saves dir")?;
        }
        if !second_dir_saves_dir.is_dir() {
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
            let (src, dest) = match second_dir_saves
                .iter()
                .enumerate()
                .find(|(_, p)| p.file_name() == path.file_name())
            {
                Some((dupe_i, dupe)) => {
                    // if it does, check which is the newest
                    let src_dest = if path
                        .metadata()
                        .context("Failed to get metadata")?
                        .modified()
                        .context("Failed to get modified time")?
                        > dupe
                            .metadata()
                            .context("Failed to get metadata")?
                            .modified()
                            .context("Failed to get modified time")?
                    {
                        (path, dupe.clone())
                    } else {
                        (dupe.clone(), path)
                    };

                    // remove dupe from list
                    second_dir_saves.remove(dupe_i);

                    src_dest
                }
                None => {
                    // if it doesn't, copy it
                    (
                        path.clone(),
                        second_dir_saves_dir
                            .join(path.file_name().context("Failed to get file name")?),
                    )
                }
            };

            // overwrite copy in second dir
            fs::copy(src, dest).context("Failed to copy save file")?;
        }

        // copy remaining files from second dir to first dir
        for path in second_dir_saves {
            fs::copy(
                &path,
                first_dir_saves_dir.join(path.file_name().context("Failed to get file name")?),
            )
            .context("Failed to copy save file")?;
        }
    }

    Ok(())
}
