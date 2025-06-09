#![allow(dead_code)]
use std::io;
use termimad::*;


/// Prompts user to continue or not, expecting 'Y' or 'N'.
/// 
/// # Arguments
/// * `skin` - Skin setting object to control rendering of output
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

/// Allows user to select an existing chat by number.
/// 
/// # Arguments
/// * `skin` - Skin setting object to control rendering of output
/// * `chats` - Vector containing the chat list to choose from
/// 
/// # Returns
/// * i64 - Selected chat ID
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

/// Prints the chat history provided.
/// 
/// # Arguments
/// * `skin` - Skin setting object to control rendering of output
/// * `messages` - Vector containing (role, content, timestamp) tuples
pub fn print_chat_history( messages: Vec<(String, String, String)>) {
    if messages.is_empty() {
        display_ai_message("No previous messages found.");
    } else {
        display_ai_message("Previous messages:");
        for (role, content, _timestamp) in messages {
            let msg = format!("[{}] {}", role, content); 
            display_ai_message(&msg);
        }
    }
}


/// Displays a message inside a simple bordered box.
/// 
/// # Arguments
/// * `message` - The plain text message to display
pub fn display_ai_message(message: &str) -> () {
    // Get terminal width (20 to 100 columns), using your terminal_size() function.
    let width = terminal_size().0.min(100).max(20) as usize;

    // Create a string to hold the box.
    let mut output = String::new();

    // Add top border: ╭──────╮
    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2)); // Fill width minus corners.
    output.push_str("╮\n");

    // Use default MadSkin (no custom styles, just plain text).
    let skin = MadSkin::default();

    // Split the message into lines and add them to the box.
    for line in message.lines() {
        let clean_line = line.trim_end(); // Remove trailing spaces.
        output.push_str("│ ");
        output.push_str(clean_line); // Add the line as-is, no Markdown.
        // Add spaces to reach the right border, but don’t go negative.
        let padding = if clean_line.len() + 4 <= width {
            width - 4 - clean_line.len() // Total width minus borders and line.
        } else {
            0 // No padding if line is too long.
        };
        output.push_str(&" ".repeat(padding));
        output.push_str(" │\n");
    }

    // Add bottom border: ╰──────╯
    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    // Print the box using default skin (plain text, no extra colors).
    skin.print_text(&output);
}