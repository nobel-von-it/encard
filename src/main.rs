use std::io::stdout;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

const PARSE_ERROR: &str = "Could not parse input";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        #[arg(short, long)]
        question: String,
        #[arg(short, long)]
        choices: Vec<String>,
        #[arg(short, long)]
        answer: usize,
    },
}

struct Screen {
    element: Element,
    score: u32,
    time: u32,
}

impl Screen {
    fn new(element: Element) -> Self {
        Self {
            element,
            score: 0,
            time: 0,
        }
    }
    fn menu(&mut self) {
        self.element.menu();
        self.score = 0;
        self.time = 0;
    }
}
#[derive(Serialize, Deserialize, Clone)]
struct Element {
    question: String,
    choices: Vec<String>,
    index: usize,
    answer: usize,
}
impl Element {
    fn default() -> Self {
        Self {
            question: String::new(),
            choices: Vec::new(),
            index: 0,
            answer: 0,
        }
    }
    fn menu(&mut self) {
        self.question = String::from("Welcome to Encard");
        self.choices = vec!["Start".to_string(), "Exit".to_string()];
        self.index = 0;
        self.answer = 0;
    }
    fn compare(&self) -> bool {
        self.answer == self.index
    }
    fn get(&self) -> usize {
        self.index
    }
}

#[derive(Serialize, Deserialize)]
struct Elements {
    elements: Vec<Element>,
}

impl Elements {
    fn load() -> Option<Element> {
        let dir_path = format!("/home/{}/.encard", whoami::username());
        let file_path = format!("{}/{}.json", dir_path, "questions");
        let mut file = std::fs::File::open(file_path.clone()).unwrap_or_else(|_| {
            let _ = std::fs::create_dir_all(dir_path);
            let _ = std::fs::File::create(file_path.clone());
            std::fs::File::open(file_path).unwrap()
        });
        let elements: Elements = serde_json::from_reader(&mut file).unwrap();

        elements
            .elements
            .get(rand::thread_rng().gen_range(0..elements.elements.len()))
            .cloned()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let args = Args::parse();
    match args.command {
        Some(Commands::Add {
            question,
            choices,
            answer,
        }) => {}
        None => {
            println!("{}", PARSE_ERROR);
            std::process::exit(0);
        }
    }
    let mut t = Terminal::new(CrosstermBackend::new(stdout()))?;

    let res = run(&mut t, &mut screen);

    disable_raw_mode()?;
    execute!(t.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    t.show_cursor()?;

    res?;
    Ok(())
}
fn run<B: Backend>(t: &mut Terminal<B>, screen: &mut Screen) -> anyhow::Result<()> {
    loop {
        t.draw(|f| screen.draw(f))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Esc => screen.state = States::Exiting,
                KeyCode::Up => screen.choice.up(),
                KeyCode::Down => screen.choice.down(),
                KeyCode::Enter => match screen.choice.get() {
                    Choice::Exit | Choice::Yes => break,
                    Choice::Start | Choice::No => screen.start(),
                    Choice::Vars(_) => {
                        screen.compare();
                        screen.update();
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}
