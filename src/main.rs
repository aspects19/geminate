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

fn prompt_for_conv(skin: &MadSkin) -> bool {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "" => return true,
            "n" => return false,
            _ => skin.print_text("Invalid input. Please enter 'Y' or 'N'."),
        }
    }
}

// fn load_old_conversation(conn: &Connection, skin: &mut MadSkin) -> Option<i64> {
//     let chats = match get_chats(conn) {
//         Ok(chats) if !chats.is_empty() => chats,
//         _ => {
//             skin.print_text("No previous conversations found.");
//             return None;
//         }
//     };

//     skin.print_text("Enter the index of the conversation you want to continue:");

//     for (index, (chat_id, created_at)) in chats.iter().enumerate() {
//         skin.print_text(format!("[{}] Chat ID: {}, Created At: {}", index, chat_id, created_at).as_str());
//     }

//     loop {
//         let mut pick = String::new();
//         std::io::stdin().read_line(&mut pick).expect("Failed to read input");

//         match pick.trim().parse::<usize>() {
//             Ok(index) if index < chats.len() => {
//                 let (selected_chat_id, _) = chats[index];
//                 skin.print_text(&format!("You selected chat ID: {}", selected_chat_id));
//                 return Some(selected_chat_id);
//             }
//             _ => skin.print_text("Invalid selection. Try again!"),
//         }
//     }
// }


#[main]
async fn main() {
    dotenv().ok();
    let conn = Connection::open("gemini.db").expect("Failed to open database");
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

    if messages.is_empty() {
        skin.print_text("No previous messages available");
    } else {
        for (role, content, _timestamp) in messages {
            skin.print_text(format!("[{}] {}", role, content).as_str());
        }
    }

    

    // skin.print_text("Hi👋 I'm Gemini. How can I help you today? (type 'exit' to leave)");

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


fn select_existing_chat(skin: &MadSkin, chats: &Vec<(i64, String)>) -> i64 {
    loop {
        let mut pick = String::new();
        io::stdin()
            .read_line(&mut pick)
            .expect("Failed to read your input");
        match pick.trim().parse::<usize>() {
            Ok(pick) if pick < chats.len() => return chats[pick].0,
            _ => skin.print_text("Invalid choice. Please enter a valid chat number."),
        }
    }
}