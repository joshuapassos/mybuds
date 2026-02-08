pub mod pages;

use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use tokio::sync::mpsc;

use crate::device::handler::PropertyStore;

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Sound,
    Gestures,
    DualConnect,
    DeviceInfo,
    Settings,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[
            Tab::Home,
            Tab::Sound,
            Tab::Gestures,
            Tab::DualConnect,
            Tab::DeviceInfo,
            Tab::Settings,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Sound => "Sound",
            Tab::Gestures => "Gestures",
            Tab::DualConnect => "Dual Connect",
            Tab::DeviceInfo => "Device Info",
            Tab::Settings => "Settings",
        }
    }

    fn index(&self) -> usize {
        Tab::all().iter().position(|t| t == self).unwrap_or(0)
    }

    fn from_index(i: usize) -> Tab {
        Tab::all().get(i).copied().unwrap_or(Tab::Home)
    }
}

/// Actions the TUI pages can emit to change device properties.
pub enum Action {
    None,
    SetProperty {
        group: String,
        prop: String,
        value: String,
    },
}

/// Shared page state: selected item index within the current page.
pub struct PageState {
    pub selected: usize,
    pub item_count: usize,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            item_count: 0,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.item_count > 0 && self.selected < self.item_count - 1 {
            self.selected += 1;
        }
    }

    pub fn clamp(&mut self) {
        if self.item_count == 0 {
            self.selected = 0;
        } else if self.selected >= self.item_count {
            self.selected = self.item_count - 1;
        }
    }
}

pub struct TuiApp {
    current_tab: Tab,
    props: PropertyStore,
    prop_tx: mpsc::Sender<(String, String, String)>,
    // Cached snapshots
    battery: HashMap<String, String>,
    anc: HashMap<String, String>,
    info: HashMap<String, String>,
    sound: HashMap<String, String>,
    actions: HashMap<String, String>,
    config: HashMap<String, String>,
    dual_connect: HashMap<String, String>,
    connected: bool,
    page_state: PageState,
}

impl TuiApp {
    fn new(props: PropertyStore, prop_tx: mpsc::Sender<(String, String, String)>) -> Self {
        Self {
            current_tab: Tab::Home,
            props,
            prop_tx,
            battery: HashMap::new(),
            anc: HashMap::new(),
            info: HashMap::new(),
            sound: HashMap::new(),
            actions: HashMap::new(),
            config: HashMap::new(),
            dual_connect: HashMap::new(),
            connected: false,
            page_state: PageState::new(),
        }
    }

    fn refresh_props(&mut self) {
        // Use try_lock to avoid blocking the UI thread
        if let Ok(store) = self.props.try_lock() {
            self.battery = store.get("battery").cloned().unwrap_or_default();
            self.anc = store.get("anc").cloned().unwrap_or_default();
            self.info = store.get("info").cloned().unwrap_or_default();
            self.sound = store.get("sound").cloned().unwrap_or_default();
            self.actions = store.get("action").cloned().unwrap_or_default();
            self.config = store.get("config").cloned().unwrap_or_default();
            self.dual_connect = store.get("dual_connect").cloned().unwrap_or_default();
            self.connected = !self.battery.is_empty();
        }
    }

    fn send_property(&self, group: &str, prop: &str, value: &str) {
        let _ = self.prop_tx.try_send((
            group.to_string(),
            prop.to_string(),
            value.to_string(),
        ));
    }

    fn switch_tab(&mut self, tab: Tab) {
        if self.current_tab != tab {
            self.current_tab = tab;
            self.page_state = PageState::new();
        }
    }

    fn next_tab(&mut self) {
        let idx = self.current_tab.index();
        let next = (idx + 1) % Tab::all().len();
        self.switch_tab(Tab::from_index(next));
    }

    fn prev_tab(&mut self) {
        let idx = self.current_tab.index();
        let prev = if idx == 0 {
            Tab::all().len() - 1
        } else {
            idx - 1
        };
        self.switch_tab(Tab::from_index(prev));
    }

    fn handle_page_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::SetProperty { group, prop, value } => {
                self.send_property(&group, &prop, &value);
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // tab bar
                Constraint::Min(0),   // page content
                Constraint::Length(1), // status bar
            ])
            .split(frame.area());

        // Tab bar
        let titles: Vec<&str> = Tab::all().iter().map(|t| t.label()).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::BOTTOM).title("MyBuds"))
            .select(self.current_tab.index())
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_widget(tabs, chunks[0]);

        // Page content
        let page_area = chunks[1];
        if !self.connected && self.current_tab != Tab::Home {
            // Show disconnected message on all non-Home tabs
            let msg = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No device connected",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "Waiting for device...",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(self.current_tab.label()));
            frame.render_widget(msg, page_area);
        } else {
            match self.current_tab {
                Tab::Home => pages::home::render(
                    frame,
                    page_area,
                    &self.battery,
                    &self.anc,
                    &self.info,
                    self.connected,
                    &mut self.page_state,
                ),
                Tab::Sound => pages::sound::render(
                    frame,
                    page_area,
                    &self.sound,
                    &self.config,
                    &mut self.page_state,
                ),
                Tab::Gestures => pages::gestures::render(
                    frame,
                    page_area,
                    &self.actions,
                    &mut self.page_state,
                ),
                Tab::DualConnect => pages::dual_connect::render(
                    frame,
                    page_area,
                    &self.dual_connect,
                    &mut self.page_state,
                ),
                Tab::DeviceInfo => pages::device_info::render(
                    frame,
                    page_area,
                    &self.info,
                ),
                Tab::Settings => pages::settings::render(
                    frame,
                    page_area,
                    &self.config,
                    &mut self.page_state,
                ),
            };
        }

        // Status bar
        let status = if self.connected {
            let model = self.info.get("device_model")
                .or_else(|| self.info.get("field_15"))
                .map(|s| s.as_str())
                .unwrap_or("FreeBuds");
            format!(" Connected: {} | q:quit Tab:switch 1-6:tab j/k:nav Enter:select h/l:cycle", model)
        } else {
            " Waiting for device... | q:quit Tab:switch".to_string()
        };
        let status_bar = Line::from(status)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(status_bar, chunks[2]);
    }

    /// Handle key events, return true if the app should quit.
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        match code {
            KeyCode::Char('q') => return true,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => return true,

            // Tab switching
            KeyCode::Char('1') => self.switch_tab(Tab::Home),
            KeyCode::Char('2') => self.switch_tab(Tab::Sound),
            KeyCode::Char('3') => self.switch_tab(Tab::Gestures),
            KeyCode::Char('4') => self.switch_tab(Tab::DualConnect),
            KeyCode::Char('5') => self.switch_tab(Tab::DeviceInfo),
            KeyCode::Char('6') => self.switch_tab(Tab::Settings),
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.prev_tab(),

            // Page navigation
            KeyCode::Up | KeyCode::Char('k') => self.page_state.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.page_state.move_down(),

            // Page actions delegated to current page
            KeyCode::Enter | KeyCode::Char(' ') => {
                let action = self.page_enter_action();
                self.handle_page_action(action);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let action = self.page_cycle_action(-1);
                self.handle_page_action(action);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let action = self.page_cycle_action(1);
                self.handle_page_action(action);
            }

            _ => {}
        }
        false
    }

    /// Enter/Space action for the current page's selected item.
    fn page_enter_action(&self) -> Action {
        match self.current_tab {
            Tab::Home => pages::home::on_enter(&self.anc, &self.page_state),
            Tab::Sound => pages::sound::on_enter(&self.sound, &self.config, &self.page_state),
            Tab::DualConnect => pages::dual_connect::on_enter(&self.dual_connect, &self.page_state),
            Tab::Settings => pages::settings::on_enter(&self.config, &self.page_state),
            _ => Action::None,
        }
    }

    /// Left/Right cycle action for the current page's selected item.
    fn page_cycle_action(&self, direction: i32) -> Action {
        match self.current_tab {
            Tab::Home => pages::home::on_cycle(&self.anc, &self.page_state, direction),
            Tab::Sound => pages::sound::on_cycle(&self.sound, &self.config, &self.page_state, direction),
            Tab::Gestures => pages::gestures::on_cycle(&self.actions, &self.page_state, direction),
            Tab::DualConnect => pages::dual_connect::on_cycle(&self.dual_connect, &self.page_state, direction),
            Tab::Settings => pages::settings::on_cycle(&self.config, &self.page_state, direction),
            _ => Action::None,
        }
    }
}

pub fn run(
    props: PropertyStore,
    prop_tx: mpsc::Sender<(String, String, String)>,
) -> Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new(props, prop_tx);
    let mut last_poll = Instant::now();

    loop {
        // Poll properties periodically
        if last_poll.elapsed() >= POLL_INTERVAL {
            app.refresh_props();
            last_poll = Instant::now();
        }

        // Draw
        terminal.draw(|f| app.draw(f))?;

        // Handle events with a timeout so we keep refreshing
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    if app.handle_key(key.code, key.modifiers) {
                        break;
                    }
                }
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
