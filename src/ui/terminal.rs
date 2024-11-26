use crate::detection::{self};
use crossterm::{
    event::{self, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io::{stdout, Result},
    path::{Path, PathBuf},
};

struct App {
    search_path: PathBuf,
    projects: Vec<detection::Project>,
    selection: ListState,
}

impl App {
    fn new(search_path: &Path) -> App {
        let mut selection = ListState::default();
        // Start with the first item selected
        selection.select(Some(0));

        App {
            search_path: search_path.to_path_buf(),
            projects: vec![],
            selection,
        }
    }

    fn detect_projects(&mut self) {
        self.projects = detection::Project::detect(self.search_path.as_path());
    }

    fn select(&self) -> bool {
        return false;
    }
    fn next(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.projects.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i == 0 {
                    self.projects.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }
}

pub fn setup() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

pub fn cleanup() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

pub fn run_app(search_path: String) -> Result<()> {
    let mut app = App::new(Path::new(&search_path));
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();

    terminal.clear()?;
    terminal.draw(|f| {
        let size = f.area();
        let block = Block::default().title("Scanning...").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    app.detect_projects();
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let items: Vec<ListItem> = app
                .projects
                .iter()
                .map(|project| {
                    let item = ListItem::new(project.name.clone().trim_matches('"').to_string())
                        .style(Style::default().fg(Color::White));
                    return item;
                })
                .collect();

            let list = List::new(items)
                .block(Block::bordered().title("Projects"))
                .highlight_style(Style::new().reversed())
                .highlight_symbol("8==D~~ ")
                .repeat_highlight_symbol(false);

            frame.render_stateful_widget(list, area, &mut app.selection);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Char('c') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            break;
                        }
                    }
                    KeyCode::Enter => {
                        if app.select() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
