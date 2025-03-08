use std::io;
use termimad::*;

pub fn prompt_for_conv(skin: &MadSkin) -> bool {
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

pub fn select_existing_chat(skin: &MadSkin, chats: &Vec<(i64, String)>) -> i64 {
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

pub fn print_chat_history(skin: &MadSkin, messages: Vec<(String, String, String)>) {
    if messages.is_empty() {
        skin.print_text("No messages in this conversation.");
    } else {
        skin.print_text("Previous messages:");
        for (role, content, _timestamp) in messages {
            skin.print_text(format!("[{}] {}", role, content).as_str());
        }
    }
}