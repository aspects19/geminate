use std::io::{self, Cursor};
use termimad::*;
use termimad::crossterm::style::{Stylize, Color::*};
use textwrap::fill;

/// Creates and returns a custom MadSkin with specific styling settings.
pub fn create_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(rgb(255, 187, 0));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
    skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
    skin.quote_mark.set_fg(Yellow);
    skin.set_global_bg(rgb(61, 74, 79));
    skin
}

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

/// Returns the terminal's width and height as a tuple (w, h).
pub fn terminal_size() -> (usize, usize) {
    match termimad::crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}

/// Displays a user message inside a simple bordered box.
/// 
/// # Arguments
/// * `message` - The plain text message to display
pub fn display_user_message(message: &str) {
    let width = terminal_size().0.min(100);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    for line in fill(message, width - 4).lines() {
        output.push_str("│ ");
        output.push_str(line);
        output.push_str(&" ".repeat(width - 4 - line.len()));
        output.push_str(" │\n");
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    println!("{}", output);
}

/// Displays an AI message with markdown formatting inside a nice box.
/// 
/// # Arguments
/// * `message` - The raw markdown-formatted string
pub fn display_ai_message(message: &str) {
    
    let width = terminal_size().0.min(100).max(20);
    println!("{}", width);
    let mut output = String::new();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let skin = create_skin();
    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);

    // Write the formatted markdown into buffer
    skin.write_text_on(&mut writer, message).unwrap();

    let formatted = String::from_utf8(buffer).unwrap();
    for line in formatted.lines() {
        let clean_line = line.trim_end();
        output.push_str("│ ");
        if clean_line.len() > width - 4 {
            // Wrap if the formatted line is too long
            for wrapped in fill(clean_line, width - 4).lines() {
                output.push_str(wrapped);
                output.push_str(&" ".repeat(width.saturating_sub(4 - wrapped.len())));
                output.push_str(" │\n│ ");
            }
            output.truncate(output.len() - 2); // Remove last extra "│ "
        } else {
            output.push_str(clean_line);
            output.push_str(&" ".repeat(width.saturating_sub(4 + clean_line.len())));
            output.push_str(" │\n");
        }
    }

    output.push_str("╰");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╯\n");

    println!("{}", output.with(rgb(255, 187, 0)));
}
