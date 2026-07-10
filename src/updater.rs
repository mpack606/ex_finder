use self_update::cargo_crate_version;
use std::error::Error;

pub fn handle_updates() -> Result<(), Box<dyn Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("mpack606")
        .repo_name("ex_finder")
        .bin_name("ex_finder")
        .show_download_progress(false)
        .no_confirm(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    if status.updated() {
        println!("Updated to version {}! Restarting...", status.version());
        std::process::exit(0);
    }
    Ok(())
}
