use std::path::Path;

use anyhow::Result;

pub fn dl_sim_steam_api_dll<P>(_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    todo!()
    // let url = "https://steamcdn-a.akamaihd.net/client/installer/steam_api.dll";
    // let mut resp = reqwest::get(url)?;
    // let mut file = File::create(path)?;

    // io::copy(&mut resp, &mut file)?;

    // Ok(())
}
