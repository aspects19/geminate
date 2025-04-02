mod db;
mod ui;

use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::{env, io, path::PathBuf};
use termimad::crossterm::style::Color::*;
use termimad::*;
use tokio::main;
use rusqlite::Connection;
use crate::db::{init_db, add_chat, add_message, get_chats, get_messages};
use crate::ui::{prompt_for_conv, select_existing_chat, print_chat_history, display_user_message, display_ai_message};

// Choose where to store database file
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


#[main]
async fn main() {
    dotenv().ok();
    // Get dtabase path depending on the environment set
    let db_path = get_data_dir();

    // Initialize a database connection
    let conn = Connection::open(db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    // Setup UI for the output
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
        display_ai_message("No previous conversation found");
        add_chat(&conn).expect("Failed to create new chat")
    } else {
        display_ai_message("Would you like to create a new conversarion? (Y/n)");

        if prompt_for_conv(&skin) {
            add_chat(&conn).expect("Failed to create a new conversation")
        } else {
            display_ai_message("Select a conversation to continue");
            for (index, (_id, created_at)) in chats.iter().enumerate() {
                display_ai_message(format!("[{}] Chat from {}", index, created_at).as_str());
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

        let user_input = user_input.trim().to_lowercase();

        if user_input.eq_ignore_ascii_case("exit") {
             break 
        };

        display_user_message(&user_input);

        let ai_response = convo.prompt(&user_input).await;

        display_ai_message(&ai_response);

        match add_message(&conn, chat_id, "user", &user_input) {
            Ok(_) => {}
            Err(e) => eprintln!("Error saving your prompt {}", e)
        }
        match add_message(&conn, chat_id, "gemini", &ai_response) {
            Ok(_) => {}
            Err(e) => eprintln!("Error saving Gemini response {}", e)
        }


    }

    display_ai_message(format!("Conversation saved under Chat ID: {}", chat_id).as_str());

    
}


// fn get_data_dir() -> PathBuf {
//     let env_mode = env::var("ENVIRONMENT").unwrap_or_else(|_| String::new());
//     if env_mode == "dev" {
//         PathBuf::from("../geminate.db")
//     } else {
//         dirs::data_local_dir()
//         .map(|p| p.join("geminate/geminate.db"))
//         .expect("Failed to get database path")
//     }
// }

// fn main() {
//     dotenv().ok();
//     let db_path = get_data_dir();
//     println!("{:?}", db_path );
// }