use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use serde::Deserialize;
use std::{
    error::Error,
    io::{self, Stdout},
    path::PathBuf,
    time::Instant,
};
use tui_input::{backend::crossterm::EventHandler, Input};

type Tui = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Deserialize)]
struct Keybind {
    keys: String,
    name: String,
    desc: String,
}

#[derive(Deserialize)]
struct Config {
    keybinds: Vec<Keybind>,
}

struct App {
    should_quit: bool,
    search_input: Input,
    items: Vec<Keybind>,
    placeholder_text: &'static str,
    first_frame_logged: bool,
    items_loaded: bool,
    scroll_offset: u16,
    content_height: u16,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            search_input: Input::default(),
            items: Vec::new(),
            placeholder_text: "Type to search keybinds",
            first_frame_logged: false,
            items_loaded: false,
            scroll_offset: 0,
            content_height: 0,
        }
    }

    // Main application loop
    fn run(&mut self, terminal: &mut Tui, profiling: bool, start: Instant) -> io::Result<()> {
        while !self.should_quit {
            self.draw(terminal);
            if profiling && !self.first_frame_logged {
                eprintln!("startup: first frame in {:.2?}", start.elapsed());
                self.first_frame_logged = true;
            }
            if !self.items_loaded {
                self.items = load_keybinds();
                self.items_loaded = true;
            }
            self.handle_events()?;
        }
        Ok(())
    }

    // Handles input events
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => self.should_quit = true,
                    KeyCode::Up => self.scroll_offset = self.scroll_offset.saturating_sub(1),
                    KeyCode::Down => self.scroll_offset = self.scroll_offset.saturating_add(1),
                    KeyCode::PageUp => {
                        self.scroll_offset = self
                            .scroll_offset
                            .saturating_sub(self.content_height.max(1));
                    }
                    KeyCode::PageDown => {
                        self.scroll_offset = self
                            .scroll_offset
                            .saturating_add(self.content_height.max(1));
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.should_quit = true
                    }
                    _ => {
                        self.search_input.handle_event(&Event::Key(key));
                        self.scroll_offset = 0;
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Tui) {
        terminal.draw(|frame| self.render_ui(frame)).unwrap();
    }

    // Renders the entire UI
    fn render_ui(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![
                Constraint::Length(1), // Title
                Constraint::Length(4), // Search
                Constraint::Length(1), // Spacer
                Constraint::Min(0),    // Content
            ])
            .split(frame.size());

        self.render_title(frame, main_layout[0]);
        self.render_search(frame, main_layout[1]);
        self.render_content(frame, main_layout[3]);
        // Footer removed intentionally.
    }

    // Renders the title bar
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let esc_text = "Esc to close";
        let esc_width = esc_text.len() as u16;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(esc_width)])
            .split(area);

        let title = Paragraph::new(Span::styled(
            "  Keybinds",
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Left);
        frame.render_widget(title, chunks[0]);

        let esc_hint = Paragraph::new(esc_text)
            .style(Style::new().fg(Color::Black))
            .alignment(Alignment::Right);
        frame.render_widget(esc_hint, chunks[1]);
    }

    // Renders the search input box
    fn render_search(&self, frame: &mut Frame, area: Rect) {
        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        let (input_text, input_style) = if self.search_input.value().is_empty() {
            (
                self.placeholder_text.to_string(),
                Style::new().fg(Color::Black),
            )
        } else {
            (
                self.search_input.value().to_string(),
                Style::new().fg(Color::White),
            )
        };

        let input_line = Line::from(Span::styled(format!(" {} ", input_text), input_style));
        let input_paragraph = Paragraph::new(Text::from(input_line)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(Color::Black)),
        );
        frame.render_widget(input_paragraph, input_area[1]);
    }

    // Renders the filtered list of keybinds
    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        if !self.items_loaded {
            let message =
                Paragraph::new("Loading keybinds...").style(Style::new().fg(Color::White));
            frame.render_widget(message, area);
            return;
        }

        self.content_height = area.height;
        let query = self.search_input.value().to_lowercase();
        let filtered_items: Vec<&Keybind> = self
            .items
            .iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&query)
                    || item.desc.to_lowercase().contains(&query)
            })
            .collect();

        if filtered_items.is_empty() {
            let message = Paragraph::new("No matches. Try a different query.")
                .style(Style::new().fg(Color::White));
            frame.render_widget(message, area);
            return;
        }

        let mut lines: Vec<Line<'static>> = Vec::new();
        let inner_width = area.width;
        for item in filtered_items {
            let key_text = format!("{} ", item.keys);
            let key_span = Span::styled(key_text.clone(), Style::new().fg(Color::White).bold());
            let name_text = item.name.clone();
            let reserved = key_text.len() + name_text.len();
            let spacer_len = if inner_width as usize > reserved {
                inner_width as usize - reserved
            } else {
                1
            };
            let name_span = Span::styled(name_text, Style::new().bold());
            lines.push(Line::from(vec![
                key_span,
                Span::raw(" ".repeat(spacer_len)),
                name_span,
            ]));
            if !item.desc.is_empty() {
                lines.push(Self::make_desc_line(&item.desc, inner_width));
            }
            lines.push(Line::from(" "));
        }

        if lines.is_empty() {
            let message = Paragraph::new("No matches. Try a different query.")
                .style(Style::new().fg(Color::White));
            frame.render_widget(message, area);
            return;
        }

        let max_scroll = lines.len().saturating_sub(area.height as usize);
        let scroll = self.scroll_offset.min(max_scroll as u16);
        let list = Paragraph::new(Text::from(lines))
            .scroll((scroll, 0))
            .style(Style::new().fg(Color::White));
        frame.render_widget(list, area);
    }

    // Creates a description line with dashes on either side
    fn make_desc_line(desc: &str, width: u16) -> Line<'static> {
        let desc_style = Style::new().fg(Color::Black);
        let inner_width = width as usize;
        let trimmed = desc.trim();

        if inner_width == 0 {
            return Line::from(Span::styled(trimmed.to_string(), desc_style));
        }

        let desc_len = trimmed.len();
        let min_needed = desc_len + 4;
        if inner_width < min_needed {
            return Line::from(Span::styled(trimmed.to_string(), desc_style));
        }

        let dash_total = inner_width - desc_len - 2;
        let left = dash_total / 2;
        let right = dash_total - left;
        let line = format!("{} {} {}", "-".repeat(left), trimmed, "-".repeat(right));
        Line::from(Span::styled(line, desc_style))
    }

    // Footer intentionally removed.
}

// Initializes terminal in alternate screen and raw mode
fn init_terminal() -> io::Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

// Restores terminal to original state
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

// Loads keybinds from user or system config, or returns defaults
fn load_keybinds() -> Vec<Keybind> {
    let user_config = xdg_config_path().map(|mut path| {
        path.push("nebula-keybind-menu");
        path.push("config.toml");
        path
    });
    let system_config = PathBuf::from("/usr/share/nebula-keybind-menu/config.toml");

    // Try user config
    if let Some(path) = user_config {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<Config>(&contents) {
                if !config.keybinds.is_empty() {
                    return config.keybinds;
                }
            }
        }
    }

    // Try system config
    if let Ok(contents) = std::fs::read_to_string(&system_config) {
        if let Ok(config) = toml::from_str::<Config>(&contents) {
            if !config.keybinds.is_empty() {
                return config.keybinds;
            }
        }
    }

    // Fallback default keybinds
    vec![
        Keybind {
            keys: "SUPER + SPACE".to_string(),
            name: "Launcher".to_string(),
            desc: "Open app launcher".to_string(),
        },
        Keybind {
            keys: "SUPER + B".to_string(),
            name: "Web Browser".to_string(),
            desc: "Open default browser".to_string(),
        },
        Keybind {
            keys: "SUPER + ENTER".to_string(),
            name: "Terminal".to_string(),
            desc: "Open terminal".to_string(),
        },
        Keybind {
            keys: "SUPER + Q".to_string(),
            name: "Close Window".to_string(),
            desc: "Close focused window".to_string(),
        },
    ]
}

// Returns the XDG config path, if available.
fn xdg_config_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path));
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".config"));
    }
    None
}

// Entry point
fn main() -> Result<(), Box<dyn Error>> {
    let profiling = std::env::var("NEBULA_KEYBIND_MENU_PROFILE").is_ok();
    let start = Instant::now();
    let mut terminal = init_terminal()?;
    if profiling {
        eprintln!("startup: terminal ready in {:.2?}", start.elapsed());
    }
    let mut app = App::new();
    if profiling {
        eprintln!("startup: app ready in {:.2?}", start.elapsed());
    }
    app.run(&mut terminal, profiling, start)?;
    restore_terminal()?;
    Ok(())
}
