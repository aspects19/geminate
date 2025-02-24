use dotenvy::dotenv;
use gemini_rs::Conversation;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
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

#[main]
async fn main() {
    dotenv().ok();
    fs::create_dir_all(CONV_DIR.as_path()).unwrap();

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

    skin.print_text("Would you like to create a new conversation? (Y/n)");

    let new_conv = loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "n" | "" => break input,
            _ => skin.print_text("Invalid input. Please enter 'Y' or 'N'."),
        }
    };

    let mut conv_uuid: Uuid = Uuid::new_v4();
    if new_conv == "y" || new_conv == "" {
        skin.print_text("Starting a new conversation...");
    } else {
        skin.print_text("Continuing with existing conversations...");
        match list_files_in_dir(CONV_DIR.as_path()) {
            None => {
                skin.print_text(format!("No old convos found in {}", CONV_DIR.as_path().display()).as_str());
                skin.print_text("Creating new conversation...");
            }
            Some(old_convos) => {
                skin.print_text("Enter valid index to choose convo: ");

                for (index, convo) in old_convos.iter().enumerate() {
                    skin.print_text(format!("[{}], {}", index, convo.to_str().unwrap()).as_str());
                }

                loop {
                    let mut pick = String::new();
                    io::stdin()
                        .read_line(&mut pick)
                        .expect("Failed to read input");

                    match pick.trim().parse::<usize>() {
                        Ok(pick) => {
                            if pick < old_convos.len() {
                                skin.print_text(
                                    format!("You selected: {}", old_convos[pick].to_str().unwrap())
                                        .as_str(),
                                );
                                convo.load(old_convos[pick].to_str().unwrap());
                                // this is dirty chaining calls tbh
                                conv_uuid = old_convos[pick]
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .strip_prefix("convo-")
                                    .unwrap()
                                    .strip_suffix(".txt")
                                    .unwrap()
                                    .parse()
                                    .unwrap();
                                break;
                            } else {
                                skin.print_text("Invalid index. Try again!");
                            }
                        }
                        Err(_) => {
                            skin.print_text("Invalid input. Please enter a number.");
                        }
                    }
                }
            }
        }
    }

    skin.print_text("Hi👋 I'm Gemini. How can I help you today?");
    loop {
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");

        let user_input = user_input.trim().to_lowercase();

        match user_input.as_str() {
            "exit" => {
                break;
            }
            _ => {}
        }

        let ai_response = convo.prompt(&user_input).await;

        skin.print_text(&ai_response);
    }

    convo.save(
        CONV_DIR
            .join(format!("convo-{}.txt", conv_uuid))
            .to_str()
            .unwrap(),
    );

    skin.print_text(format!("Convo saved with uuid: {}", conv_uuid).as_str());
}
