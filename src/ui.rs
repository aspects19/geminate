use std::io;
use termimad::*;
use termimad::crossterm::style::Stylize;
use termimad::crossterm::style::Color::*;


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


    /// Iterates over user input to return Either true or false if a valid input is provided
    /// 
    /// # Arguments
    /// 
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

/// Picks a chat based on user input
    /// 
    /// # Arguments 
    /// 
    /// * `skin` - Skin setting object to control rendering of output
    /// * `chats` - Vector containing the chat list to choose from
    /// 
    /// # Returns
    /// 
    /// A usize integer of the chat number
    /// 
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

/// Prints the whole chat history of a provided message vector list
    /// 
    /// # Arguments 
    /// 
    /// * `skin` - Skin setting object to control rendering of output
    /// * `messages` - Message vector list to Print its contents
    /// 
    /// # Return
    /// 
    /// Null

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

pub fn terminal_size() -> (usize, usize) {
    match termimad::crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}

pub fn display_user_message(message: &str) {
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

pub fn display_ai_message(message: &str) {
    let width: usize = terminal_size().0.min(100);
    let mut output = String::new();

    let skin = MadSkin::default();

    output.push_str("\n╭");
    output.push_str(&"─".repeat(width - 2));
    output.push_str("╮\n");

    let formatted_text = String::new();
    skin.write_text(message)
        .expect("failed to format text");

    let wrapped_text = textwrap::fill(&formatted_text, width - 4);
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

