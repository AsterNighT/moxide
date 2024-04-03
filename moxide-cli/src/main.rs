mod app;
use app::App;
use color_eyre::eyre::Result;
use rustyline::DefaultEditor;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
fn init() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(LevelFilter::INFO)
        .init();
    if !is_elevated::is_elevated() {
        tracing::warn!("The program is not run as elevated. Most functions will be forbidden.")
    }
    Ok(())
}

fn main() -> Result<()> {
    init()?;
    let mut app = App::new();
    start_loop(&mut app)?;
    Ok(())
}

fn start_loop(app: &mut App) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        let input_line = rl.readline("> ")?;
        match app.handle_input(&input_line.trim()) {
            Ok(output) => println!("{}", output),
            Err(e) => eprintln!("{}", e),
        }
    }
}
