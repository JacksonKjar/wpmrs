use crossterm::{
    event::{EnableMouseCapture, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, error, info};
use rand::prelude::*;
use std::{
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
};
use wpm::{loggers::DeferredLogger, webscraping::typeracerdata::TextRow, TypeRacePrompt};

use crossterm::event::{read, Event};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Margin,
    Frame, Terminal,
};

fn main() {
    static LOGGER: DeferredLogger = DeferredLogger::new();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    debug!("Parsing args");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|a| a.as_str()).collect();
    let res = match args.as_slice() {
        ["fetch-prompts"] => fetch_prompts("prompts.json"),
        ["fetch-prompts", filename] => fetch_prompts(filename),
        [] => play(),
        _ => Err("Invalid arguments".into()),
    };
    if let Err(err) = res {
        error!("Error: {err}");
    }
    log::logger().flush();
}

fn fetch_prompts(filename: &str) -> Result<(), Box<dyn Error>> {
    static URL: &str = "https://typeracerdata.com/texts?texts=full&sort=relative_average";
    info!("Fetching prompts from {URL}");
    let resp = reqwest::blocking::get(URL)?.text()?;
    info!("Parsing response");
    let table: Vec<_> = TextRow::parse_table(&resp).collect();
    info!("Successfully parsed {} prompts", table.len());
    let file = File::create(filename)?;
    let buf = BufWriter::new(file);
    info!("Writing prompts to {filename}");
    serde_json::to_writer_pretty(buf, &table)?;
    Ok(())
}

fn play() -> Result<(), Box<dyn Error>> {
    info!("Reading prompts");
    let file = File::open("prompts.json")?;
    let mut prompts: Vec<TextRow> = serde_json::from_reader(BufReader::new(file))?;
    prompts.shuffle(&mut rand::thread_rng());
    info!("Setting up terminal for TUI");
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    run(Terminal::new(backend)?, prompts)?;
    info!("Restoring terminal");
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run<D: Backend>(
    mut terminal: Terminal<D>,
    prompts: impl IntoIterator<Item = TextRow>,
) -> Result<(), Box<dyn Error>> {
    for mut prompt in prompts.into_iter().map(|tr| TypeRacePrompt::new(tr.text)) {
        info!("Starting race");
        terminal.draw(|f| redraw(f, prompt.clone()))?;
        while !prompt.is_complete() {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    info!("Kill signal received: Exiting race");
                    return Ok(());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    info!("Skipping prompt");
                    break;
                }
                Event::Key(ke) => {
                    prompt.apply_key(ke);
                    terminal.draw(|f| redraw(f, prompt.clone()))?;
                }
                _ => (),
            }
        }
        if prompt.is_complete() {
            info!("Race completed");
        } else {
            info!("Race failed")
        }
    }
    Ok(())
}

fn redraw<B: Backend>(frame: &mut Frame<B>, prompt: TypeRacePrompt) {
    let size = frame.size();
    let margin = Margin {
        vertical: 5,
        horizontal: 5,
    };
    frame.render_widget(&prompt, size.inner(&margin));
}
