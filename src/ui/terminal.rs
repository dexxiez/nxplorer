use crate::detection::{project::ProjectType, Project};
use crossterm::{
    event::{self},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{prelude::*, widgets::*};
use std::{
    io::{stdout, Result},
    path::{Path, PathBuf},
};
use tui_textarea::{Input, Key, TextArea};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct CommandEntry {
    project_type: ProjectType,
    framework_name: Option<String>,
    project_name: String,
    command: String,
    subcommand: Option<String>,
}

impl CommandEntry {
    fn display_string(&self) -> String {
        let project_type_str = match self.project_type {
            ProjectType::Library => "lib",
            ProjectType::Application => "app",
        };

        let type_display = if let Some(framework) = &self.framework_name {
            format!("{}:{}", project_type_str, framework)
        } else {
            project_type_str.to_string()
        };

        if let Some(subcommand) = &self.subcommand {
            format!(
                "[{}] {}:{}:{}",
                type_display, self.project_name, self.command, subcommand
            )
        } else {
            format!("[{}] {}:{}", type_display, self.project_name, self.command)
        }
    }

    fn project_type_display(&self) -> String {
        match self.project_type {
            ProjectType::Library => "lib".to_string(),
            ProjectType::Application => "app".to_string(),
        }
    }

    fn command_display(&self) -> String {
        if let Some(subcommand) = &self.subcommand {
            format!("{}:{}", self.command, subcommand)
        } else {
            self.command.clone()
        }
    }

    fn to_nx_command(&self) -> String {
        if let Some(subcommand) = &self.subcommand {
            format!("{}:{}:{}", self.project_name, self.command, subcommand)
        } else {
            format!("{}:{}", self.project_name, self.command)
        }
    }
}

struct App {
    search_path: PathBuf,
    projects: Vec<Project>,
    all_commands: Vec<CommandEntry>,
    display_commands: Vec<CommandEntry>,
    selection: ListState,
}

impl App {
    fn new(search_path: &Path) -> App {
        let mut selection = ListState::default();
        selection.select(Some(0));

        App {
            search_path: search_path.to_path_buf(),
            projects: vec![],
            all_commands: vec![],
            display_commands: vec![],
            selection,
        }
    }

    fn detect_projects(&mut self) {
        self.projects = Project::detect(self.search_path.as_path());
    }

    fn select(&self) -> bool {
        if let Some(i) = self.selection.selected() {
            if i < self.display_commands.len() {
                let selected_command = &self.display_commands[i];
                let _ = cleanup();

                let status = std::process::Command::new("nx")
                    .arg("run")
                    .arg(selected_command.to_nx_command())
                    .status()
                    .expect("Failed to execute nx command");

                if status.success() {
                    std::process::exit(0);
                }

                println!("Command exited with status: {}", status);
                return true;
            }
        }
        false
    }

    fn next(&mut self) {
        let i = match self.selection.selected() {
            Some(i) => {
                if i >= self.display_commands.len() - 1 {
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
                    self.display_commands.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection.select(Some(i));
    }

    fn prep_for_matching(input: &str) -> String {
        input
            .replace("[", "")
            .replace("]", "")
            .replace(":", "")
            .replace("-", "")
            .replace(" ", "")
            .to_lowercase()
    }

    fn filter_commands(&mut self, search: &str) {
        let matcher = SkimMatcherV2::default();
        let prepped_search = Self::prep_for_matching(search);

        let mut matched_commands: Vec<(i64, CommandEntry)> = self
            .all_commands
            .iter()
            .filter_map(|cmd| {
                let display = cmd.display_string();
                let mut prepped_cmd = Self::prep_for_matching(&display);
                let reversed_words_command =
                    display.split_whitespace().rev().collect::<Vec<&str>>();
                prepped_cmd = prepped_cmd + " " + &reversed_words_command.join(" ");

                matcher
                    .fuzzy_match(&prepped_cmd, &prepped_search)
                    .map(|score| (score, cmd.clone()))
            })
            .collect();

        matched_commands.sort_by(|a, b| b.0.cmp(&a.0));

        self.display_commands = matched_commands.into_iter().map(|(_, cmd)| cmd).collect();

        if self.display_commands.is_empty() {
            self.selection.select(None);
        } else {
            self.selection.select(Some(0));
        }
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

fn nx_reset() {
    std::process::Command::new("nx")
        .arg("reset")
        .status()
        .expect("Failed to execute nx reset");

    let _ = cleanup();
    std::process::exit(0);
}

pub fn run_app(search_path: String) -> Result<()> {
    let mut app = App::new(Path::new(&search_path));
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    let mut textarea = TextArea::default();
    terminal.clear()?;
    terminal.draw(|f| {
        let size = f.area();
        let block = Block::default().title("Scanning...").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    app.detect_projects();
    app.display_commands = construct(&app.projects);
    app.all_commands = app.display_commands.clone();
    terminal.clear()?;
    if app.display_commands.is_empty() {
        println!("No projects found in the specified path");
        return Ok(());
    }
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(8),
                    Constraint::Percentage(87),
                    Constraint::Percentage(5),
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
                        app.display_commands.len()
                    ),
                    Style::default().fg(Color::Gray),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![Span::styled(
                    "arrow keys to navigate, enter/tab to select, q / esc to quit",
                    Style::default().fg(Color::Yellow),
                )])
                .alignment(Alignment::Center),
                Line::from(vec![Span::styled(
                    "ctrl + r to reset nx",
                    Style::default().fg(Color::Yellow),
                )])
                .alignment(Alignment::Center),
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
                .display_commands
                .iter()
                .map(|cmd| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:20}", cmd.project_type_display()),
                            Style::default().fg(Color::LightRed),
                        ),
                        Span::styled(
                            format!(
                                "{:20}",
                                cmd.framework_name.clone().unwrap_or("".to_string())
                            ),
                            Style::default().fg(Color::LightBlue),
                        ),
                        Span::styled(
                            format!("{:40}", cmd.project_name),
                            Style::default().fg(Color::LightGreen),
                        ),
                        Span::styled(
                            cmd.command_display(),
                            Style::default().fg(Color::LightYellow),
                        ),
                    ]))
                })
                .collect();

            // Making the selection highlight more visible with a darker background
            let list = List::new(items)
                .block(Block::bordered().title("Projects"))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White) // Makes the text really pop when selected
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_stateful_widget(list, list_area, &mut app.selection);

            // For the list and textbox, we'll split vertically with percentage constraints

            let search_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(layout[2])[1];

            let border_color = match app.display_commands.len() {
                0 => Color::Red,
                _ => Color::Gray,
            };

            // Style the textarea directly
            textarea.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );

            // Render it directly - no .widget() needed
            frame.render_widget(&textarea, search_area);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(10))? {
            match event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Char('r'),
                    ctrl: true,
                    ..
                } => nx_reset(),
                Input {
                    key: Key::Char('c'),
                    ctrl: true,
                    ..
                } => break,
                Input { key: Key::Down, .. } => app.next(),
                Input { key: Key::Up, .. } => app.previous(),
                Input { key: Key::Tab, .. } => {
                    if app.select() {
                        break;
                    }
                }
                Input {
                    key: Key::Enter, ..
                } => {
                    if app.select() {
                        break;
                    }
                }
                input => {
                    textarea.input(input);
                    let search_text = textarea.lines()[0].to_string();
                    app.filter_commands(&search_text);
                }
            }
        }
    }

    Ok(())
}

fn construct(projects: &Vec<Project>) -> Vec<CommandEntry> {
    let mut cmds: Vec<CommandEntry> = Vec::new();

    for project in projects {
        let framework_name = project.framework.map(|f| f.name.to_string());

        // Add regular tasks
        for task in &project.tasks {
            // Add main command
            cmds.push(CommandEntry {
                project_type: project.project_type.clone(),
                framework_name: framework_name.clone(),
                project_name: project.name.clone(),
                command: task.command.clone(),
                subcommand: None,
            });

            // Add subcommands
            for subcmd in &task.subcommands {
                cmds.push(CommandEntry {
                    project_type: project.project_type.clone(),
                    framework_name: framework_name.clone(),
                    project_name: project.name.clone(),
                    command: task.command.clone(),
                    subcommand: Some(subcmd.clone()),
                });
            }
        }

        // Add framework commands
        if let Some(framework) = &project.framework {
            for cmd in framework.commands {
                cmds.push(CommandEntry {
                    project_type: project.project_type.clone(),
                    framework_name: Some(framework.name.to_string()),
                    project_name: project.name.clone(),
                    command: cmd.to_string(),
                    subcommand: None,
                });
            }
        }
    }

    // Remove duplicates based on the actual command that would be run
    cmds.sort_by(|a, b| a.to_nx_command().cmp(&b.to_nx_command()));
    cmds.dedup_by(|a, b| a.to_nx_command() == b.to_nx_command());

    cmds
}
