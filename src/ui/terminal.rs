use crate::detection::{project::Framework, Project};
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

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct App {
    search_path: PathBuf,
    projects: Vec<Project>,
    commands: Vec<String>,
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
            commands: vec![],
            selection,
        }
    }

    fn detect_projects(&mut self) {
        self.projects = Project::detect(self.search_path.as_path());
    }

    fn select(&self) -> bool {
        if let Some(i) = self.selection.selected() {
            if i < self.commands.len() {
                let selected_command = &self.commands[i];
                // Exit raw mode and leave alternate screen before executing command
                let _ = cleanup();

                // Split the command into parts
                let parts: Vec<&str> = selected_command.split(':').collect();
                if parts.len() >= 2 {
                    // Construct and execute nx command
                    let status = std::process::Command::new("nx")
                        .arg("run")
                        .arg(selected_command.replace(':', ":"))
                        .status()
                        .expect("Failed to execute nx command");

                    if status.success() {
                        std::process::exit(0);
                    }

                    println!("Command exited with status: {}", status);
                }
                return true;
            }
        }
        false
    }

    fn next(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.commands.len() - 1 {
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
                    self.commands.len() - 1
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
    app.commands = construct(&app.projects);
    terminal.clear()?;
    if app.commands.is_empty() {
        println!("No projects found in the specified path");
        return Ok(());
    }
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // Combined title block + top padding
                    Constraint::Min(0),    // List
                ])
                .split(area);

            // Combined title block
            let title_block = Block::default()
                .title(Line::from(vec![])) // Add top padding line
                .title_alignment(Alignment::Center);

            let titles = Paragraph::new(vec![
                Line::from(vec![Span::styled(
                    format!("{} {}", NAME, VERSION),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    format!(
                        "{} projects with {} tasks",
                        app.projects.len(),
                        app.commands.len()
                    ),
                    Style::default().fg(Color::Gray),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![Span::styled(
                    "j / k or arrow keys to navigate, Enter to select, q / esc to quit",
                    Style::default().fg(Color::DarkGray),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![]),
            ]);

            frame.render_widget(title_block, layout[0]);
            frame.render_widget(titles, layout[0]);

            // List in center column
            let list_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(layout[1])[1];

            let items: Vec<ListItem> = app
                .commands
                .clone()
                .into_iter()
                .map(|cmd| {
                    ListItem::new(cmd.replace('"', "").to_string())
                        .style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(Block::bordered())
                .highlight_style(Style::new().magenta());

            frame.render_stateful_widget(list, list_area, &mut app.selection);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(50))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
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

fn dedupe<T: PartialEq>(v: &mut Vec<T>) {
    let mut i = 0;
    while i < v.len() {
        if v[i..].iter().skip(1).any(|x| x == &v[i]) {
            v.drain(i..i + 1);
        } else {
            i += 1;
        }
    }
}

fn construct(projects: &Vec<Project>) -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    projects.iter().for_each(|project| {
        let framework = project.framework.clone().unwrap_or_else(|| {
            return Framework {
                name: "",
                commands: &[],
                identity_files: &[],
                identity_keywords: &[],
            };
        });

        if !project.tasks.is_empty() {
            project.tasks.iter().for_each(|task| {
                cmds.push(format!("{}:{:?}", project.name, task.command));
                if task.subcommands.is_empty() {
                    return;
                }

                task.subcommands.iter().for_each(|subcmd| {
                    cmds.push(format!("{}:{}:{}", project.name, task.command, subcmd));
                });
            });
        }

        if framework.commands.is_empty() {
            return;
        }

        framework.commands.iter().for_each(|cmd| {
            cmds.push(format!("{}:{}", project.name, cmd));
        });
    });
    dedupe(&mut cmds);
    return cmds;
}
