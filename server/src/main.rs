use std::error::Error;

mod game;

fn main() -> Result<(), Box<dyn Error>> {
    game::run_game_server()?;
    Ok(())
}
