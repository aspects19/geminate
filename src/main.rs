#![allow(dead_code)]
mod db;
mod ui;
#[allow(unused)]
use crate::db::{init_db, add_chat, add_message, get_chats, get_messages};
#[allow(unused)]
use crate::ui::{prompt_for_conv, select_existing_chat, print_chat_history, display_user_message, display_ai_message};

use std::{env, io, path::PathBuf};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use dotenvy::dotenv;
use gemini_rs::Conversation;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Widget, Wrap};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::{Line, Text},
    DefaultTerminal, Frame,
};
use rusqlite::Connection;

fn get_data_dir() -> PathBuf {
    let env_mode = env::var("ENVIRONMENT").unwrap_or_else(|_| String::new());
    if env_mode == "dev" {
        PathBuf::from("./geminate.db")
    } else {
        dirs::data_local_dir()
            .map(|p| p.join("geminate/geminate.db"))
            .expect("Failed to get database path")
    }
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    chat_id: i64,
    messages: Vec<(String, String)>,
    input: String,
    scroll: u16,
    convo: Conversation,
    conn: Connection,
    state: AppState,
    chat_selection: ListState,
    chats: Vec<(i64, String)>,
}

#[derive(Debug, PartialEq)]
enum AppState {
    SelectingChat,
    Chatting,
    CreatingNewChat,
}

impl App {
    pub fn new(conn: Connection, convo: Conversation, chats: Vec<(i64, String)>) -> Self {
        let state = if chats.is_empty() {
            AppState::CreatingNewChat
        } else {
            AppState::SelectingChat
        };
        App {
            exit: false,
            chat_id: 0, // Will be set after selection or creation
            messages: Vec::new(),
            input: String::new(),
            scroll: 0,
            convo,
            conn,
            state,
            chat_selection: ListState::default(),
            chats,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.state {
            AppState::SelectingChat | AppState::CreatingNewChat => {
                self.render_chat_selection(frame, frame.area());
            }
            AppState::Chatting => {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1), // Chat history
                        Constraint::Length(3), // Input area
                    ])
                    .split(frame.area());

                self.render_chat_history(frame, layout[0]);
                self.render_input(frame, layout[1]);
            }
        }
    }

    fn render_chat_selection(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(Line::from(" Select Chat (Enter: Select, n: New Chat, q: Quit) ").bold())
            .border_set(border::PLAIN);

        let items: Vec<ListItem> = self
            .chats
            .iter()
            .map(|(_id, created_at)| ListItem::new(format!("Chat from {}", created_at)))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.chat_selection);
    }

    fn render_chat_history(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(Line::from(" Chat History ").bold())
            .border_set(border::PLAIN);

        let mut lines = vec![];
        for (role, content) in &self.messages {
            let prefix = if role == "user" { "You: " } else { "Gemini: " };
            let styled_prefix = if role == "user" {
                prefix.blue().bold()
            } else {
                prefix.green().bold()
            };
            lines.push(Line::from(vec![styled_prefix, content.as_str().into()]));
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(Line::from(" Input (Press 'q' to quit, 'Enter' to send) ").bold())
            .border_set(border::PLAIN);

        let paragraph = Paragraph::new(self.input.as_str())
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event).await;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            AppState::SelectingChat => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('n') => {
                    self.state = AppState::CreatingNewChat;
                    self.create_new_chat().await;
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.chat_selection.selected() {
                        self.chat_id = self.chats[selected].0;
                        self.messages = get_messages(&self.conn, self.chat_id)
                            .unwrap_or_else(|_| Vec::new())
                            .into_iter()
                            .map(|(_id, role, content)| (role, content))
                            .collect();
                        self.state = AppState::Chatting;
                    }
                }
                KeyCode::Up => {
                    let selected = self
                        .chat_selection
                        .selected()
                        .map(|i| i.saturating_sub(1))
                        .unwrap_or(0);
                    self.chat_selection.select(Some(selected));
                }
                KeyCode::Down => {
                    let selected = self
                        .chat_selection
                        .selected()
                        .map(|i| i.saturating_add(1).min(self.chats.len() - 1))
                        .unwrap_or(0);
                    self.chat_selection.select(Some(selected));
                }
                _ => {}
            },
            AppState::CreatingNewChat => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                _ => {}
            },
            AppState::Chatting => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Enter => self.send_message().await,
                KeyCode::Char(c) => self.input.push(c),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Up => {
                    if self.scroll > 0 {
                        self.scroll -= 1;
                    }
                }
                KeyCode::Down => {
                    self.scroll += 1;
                }
                _ => {}
            },
        }
    }

    async fn create_new_chat(&mut self) {
        self.chat_id = add_chat(&self.conn).expect("Failed to create new chat");
        self.messages = Vec::new();
        self.state = AppState::Chatting;
    }

    async fn send_message(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        let user_input = self.input.clone();
        self.messages.push(("user".to_string(), user_input.clone()));
        add_message(&self.conn, self.chat_id, "user", &user_input).unwrap_or_else(|e| {
            eprintln!("Error saving user message: {}", e);
        });

        let ai_response = self.convo.prompt(&user_input).await; // Await async call
        self.messages.push(("gemini".to_string(), ai_response.clone()));
        add_message(&self.conn, self.chat_id, "gemini", &ai_response).unwrap_or_else(|e| {
            eprintln!("Error saving AI response: {}", e);
        });

        self.input.clear();
        self.scroll = self.messages.len() as u16; // Scroll to bottom
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Geminate ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title(instructions.right_aligned())
            .border_set(border::PLAIN);

        let counter_text = Text::from(vec![Line::from(vec!["Value: ".into()])]);

        Paragraph::new(counter_text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let db_path = get_data_dir();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    let convo = Conversation::new(
        env::var("GEMINIAI_API").expect("Gemini API not set"),
        "gemini-1.5-flash".to_string(),
    );

    let chats = get_chats(&conn).expect("Failed to retrieve chats");

    let mut terminal = ratatui::init();
    let mut app = App::new(conn, convo, chats);
    let result = app.run(&mut terminal).await; // Await async run
    ratatui::restore();
    result
}