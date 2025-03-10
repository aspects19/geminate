mod db;
mod api;
mod ui;

use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::{env, io};
use termimad::crossterm::style::Color::*;
use termimad::*;
use tokio::main;
use rusqlite::Connection;
use crate::db::{init_db, add_chat, add_message, get_chats, get_messages};
use crate::ui::{prompt_for_conv, select_existing_chat, print_chat_history};



pub fn list_chats(conn: &Connection) -> Option<Vec<(i64, String)>> {
    match get_chats(conn) {
        Ok(chats) if !chats.is_empty() => Some(chats),
        Ok(_) => None, // No chats found
        Err(e) => {
            eprintln!("Failed to retrieve chats: {}", e);
            None
        }
    }
}


#[main]
async fn main() {
    dotenv().ok();
    // Create an OS independent database storage path
    let db_dir = dirs::data_local_dir()
        .expect("Failed to get local data directory")
        .join("geminate.db");
    let db_path = db_dir.join("geminate.db");

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


    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim().to_lowercase();

        if user_input.eq_ignore_ascii_case("exit") {
             break 
        };

        let ai_response = convo.prompt(&user_input).await;

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


