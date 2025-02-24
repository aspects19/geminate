use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::{env, io};
use termimad::crossterm::style::Color::*;
use termimad::*;
use tokio::main;

#[main]
async fn main() {
    dotenv().ok();

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

    skin.print_text("Hi👋 I'm Gemini. How can I help you today?");

    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim();

        let ai_response = convo.prompt(user_input).await;

        skin.print_text(&ai_response);
    }
}
