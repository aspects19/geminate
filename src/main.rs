use crossterm::style::Stylize;
mod db;
mod ui;
use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::{env, io};
use termimad::crossterm::style::Color::*;
use termimad::*;
use tokio::main;
use uuid::Uuid;

static CONV_DIR: LazyLock<PathBuf, fn() -> PathBuf> = LazyLock::new(|| {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("gemini-chat/convos")
});

fn display_logo() {
    println!("{}", "\n");
    println!("    ██████╗ ███████╗███╗   ███╗██╗███╗   ██╗ █████╗ ████████╗███████╗");
    println!("   ██╔════╝ ██╔════╝████╗ ████║██║████╗  ██║██╔══██╗╚══██╔══╝██╔════╝");
    println!("   ██║  ███╗█████╗  ██╔████╔██║██║██╔██╗ ██║███████║   ██║   █████╗  ");
    println!("   ██║   ██║██╔══╝  ██║╚██╔╝██║██║██║╚██╗██║██╔══██║   ██║   ██╔══╝  ");
    println!("   ╚██████╔╝███████╗██║ ╚═╝ ██║██║██║ ╚████║██║  ██║   ██║   ███████╗");
    println!("    ╚═════╝ ╚══════╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝   ╚══════╝");
    println!("{}", "\n");
}

fn display_user_message(message: &str) {
    let width = terminal_size().0.min(100);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let wrapped_text = textwrap::fill(message, width - 4);
    for line in wrapped_text.lines() {
        output.push_str("│ ");
        output.push_str(line);
        output.push_str(&" ".repeat(width - 4 - line.len()));
        output.push_str(" │\n");
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    // blue
    println!("{}", output.with(rgb(0, 120, 255)));
}

fn display_ai_message(message: &str) {
    let width = terminal_size().0.min(100);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let wrapped_text = textwrap::fill(message, width - 4);
    for line in wrapped_text.lines() {
        output.push_str("│ ");
        output.push_str(line);
        output.push_str(&" ".repeat(width - 4 - line.len()));
        output.push_str(" │\n");
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    // yellow
    println!("{}", output.with(rgb(255, 187, 0)));
}

fn terminal_size() -> (usize, usize) {
    match termimad::crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}

fn list_files_in_dir(dir: &Path) -> Option<Vec<PathBuf>> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let old_convos: Vec<_> = entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if path.is_file() {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();
            (!old_convos.is_empty()).then_some(old_convos)
        }
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            None
        }
    }
}

use rusqlite::Connection;
use crate::db::{init_db, add_chat, add_message, get_chats, get_messages};
use crate::ui::{prompt_for_conv, select_existing_chat, print_chat_history};


#[main]
async fn main() {
    dotenv().ok();
    // Create an OS independent database storage path
    let db_dir = dirs::data_local_dir()
        .expect("Failed to get local data directory")
        .join("geminate.db");
    let db_path = db_dir.join("geminate.db");

    display_logo();

    // Initialize a database connection
    let conn = Connection::open(db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    
    let mut skin = MadSkin::default();
    skin.set_headers_fg(rgb(255, 187, 0));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
    skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
    skin.quote_mark.set_fg(Yellow);
    skin.set_global_bg(rgb(61, 74, 79));

    let mut convo = Conversation::new(
        env::var("GEMINIAI_API").expect("Gemini API not set"),
        "gemini-1.5-flash".to_string(),
    );

    // A vector list of all chats
    let chats = get_chats(&conn).expect("Failed to retrieve chats");

    let chat_id = if chats.is_empty() {
        skin.print_text("No previous conversation found");
        add_chat(&conn).expect("Failed to create new chat")
    } else {
        skin.print_text("Would you like to create a new conversarion? (Y/n)");

        if prompt_for_conv(&skin) {
            add_chat(&conn).expect("Failed to create a new conversation")
        } else {
            skin.print_text("Select a conversation to continue");
            for (index, (_id, created_at)) in chats.iter().enumerate() {
                skin.print_text(format!("[{}] Chat from {}", index, created_at).as_str());
            }
            select_existing_chat(&skin, &chats)
        }
    };

    let messages = get_messages(&conn, chat_id).expect("Failed to retrive past messages");

    print_chat_history(&skin, messages);


    display_ai_message("Hi👋 I'm Gemini. How can I help you today? (type 'exit' to leave)");

    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim();

        if user_input.to_lowercase() == "exit" {
            break;
        }
        if user_input.eq_ignore_ascii_case("exit") {
             break 
        };

        display_user_message(user_input);

        let ai_response = convo.prompt(user_input).await;

        display_ai_message(&ai_response);
    }

    let conv_path = CONV_DIR.join(format!("convo-{}.txt", conv_uuid));
        match add_message(&conn, chat_id, "user", &user_input) {
            Ok(_) => {}
            Err(e) => eprintln!("Error saving your prompt {}", e)
        }
        match add_message(&conn, chat_id, "gemini", &ai_response) {
            Ok(_) => {}
            Err(e) => eprintln!("Error saving Gemini response {}", e)
        }


    }

    skin.print_text(format!("Conversation saved under Chat ID: {}", chat_id).as_str());

    
}


