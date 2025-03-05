# Geminate 🌱
![Rust](https://img.shields.io/badge/Rust-v1.76.0-orange?style=flat&logo=rust&logoColor=white)
![Termimad](https://img.shields.io/badge/Termimad-v0.31.2-blue?style=flat&logo=terminal&logoColor=white)
![Gemini-rs](https://img.shields.io/badge/Gemini--rs-v0.4.2-purple?style=flat&logo=google-gemini&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-v1.43.0-red?style=flat&logo=lightning&logoColor=white)

## Introduction 

**Geminate** is a lightweight Rust based CLI tool to interact with **Google Gemini** in your terminal. 

## Features
This project has the following features.

- [x] Back and forth communication with Gemini API.
- [x] Fast and efficient as it is written in Rust.
- [x] Memory so this binary can remember your past prompts within a chat.
- [ ] Eye catching UI.
- [ ] Setting of GEMINI_API in system environment.
- [ ] Publishing to *RPM Fusion* and *scoop*.

## Installation 

To install the project you need to clone download the binary from the [releases](https://github.com/aspects19/geminate/releases/tag/bin) page.
There are future plans to publish it to Package managers.

### Build from source

To build this project from source do the following
1. Ensure you have **Rust** installed by running `rustc --version` if not head to [Rust Docs](https://www.rust-lang.org/tools/install) to install it.
2. Clone the repository of this project.
    ``` sh
    git clone https://github.com/aspects19/geminate
    ```
3. Navigate to the project.
    ``` sh
    cd geminate
    ```
4. rename `.env.example ` to ` .env ` and replace ` ` with a Gemini API key from [Google Labs](https://aistudio.google.com/apikey)
5. Build the project using.
    ``` sh
    cargo build --release
    ```

This will create a binary in `./target/release/` that can run as a standalone app.

## Contributions

If You wish to make contributions to this project such as reporting issues and bugs, fixing them and adding features, take a look at [contribution guide](https://github.com/aspects19/geminate/blob/main/CONTRIBUTING.md)

## License

This project is licensed under the MIT License – see the [LICENSE](https://github.com/aspects19/geminate/blob/main/LICENSE) file for details.

##

Thanks for checking out the Geminate! Feel free to open issues, fork the repository, or contribute to making it even better!
