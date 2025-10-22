mod cursor;
mod editor;
mod features;
mod input;
mod settings;
use color_eyre::Result;
use crossterm::{ExecutableCommand, cursor::SetCursorStyle, style};
use editor::Document;
use input::handle_input;
use ratatui::DefaultTerminal;
mod files;
mod highlight;
mod snippets;
use snippets::function;

// f -> p -> 1 number -> value // change function argument name
// f -> p -> 2 number -> value // change funciton name
// f -> p -> del
//
// f -> a -> number (none if only one) / type -> value

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // let file = read_file("Cargo.toml").unwrap();
    let fn_snippet = &mut function::Snippet::default();
    let document = &mut Document::new("");
    terminal.backend_mut().execute(SetCursorStyle::SteadyBlock);

    loop {
        terminal.draw(|f| document.ui(f, fn_snippet))?;
        // pass viewport height to input handler so scrolling/clamping can be computed
        let viewport_height = Some((terminal.size()?.height as usize).saturating_sub(0)); // editor height used in ui
        if handle_input(document, fn_snippet, &mut terminal, viewport_height)? {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}
