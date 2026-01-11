use color_eyre;

mod app;
mod errors;
mod repository;
mod structs;
mod tui;
mod utils;
mod widgets;

use structs::{App, UserConfig};

fn main() -> color_eyre::Result<()> {
  errors::install_hooks()?;
  let mut terminal = tui::init()?;
  let user_config: UserConfig = confy::load("tomato", "config")
    .expect("Error when loading the config file");
  App::new(&user_config).run(&mut terminal)?;
  tui::restore()?;
  Ok(())
}
