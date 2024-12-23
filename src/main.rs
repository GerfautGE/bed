use crop::Rope;
use regex::Regex;
use std::string::ToString;
use std::{env, io::Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme, ThemeSet, ThemeSettings};
use syntect::parsing::SyntaxSet;


/**
 * bed - a basic editor like ed but with a modern interface
 * Author: Luc Videau
 */

macro_rules! hex2color {
    ($hex:expr) => {
        Color {
            r: u8::from_str_radix(&$hex[1..3], 16).unwrap(),
            g: u8::from_str_radix(&$hex[3..5], 16).unwrap(),
            b: u8::from_str_radix(&$hex[5..7], 16).unwrap(),
            a: 255,
        }
    };
}

#[derive(Debug)]
struct Range {
    start: usize,
    end: usize,
}

enum BedCommand {
    Quit,
    Print { range: Range },
    NPrint { range: Range },
    Move { line: usize },
    Change,
    None,
}

struct BedState {
    content: Rope,
    current_line: usize,
}

fn parse_command(input: &str, current_line: usize, max_line: usize) -> BedCommand {
    let input = input.trim();

    /* Regular Expressions */
    let quit_re = Regex::new(r"^(q|quit)$").unwrap();
    let print_re = Regex::new(r"^(\d+)?,?(\s)?(\d+)?(\s)?[pn]$").unwrap();
    let move_re = Regex::new(r"^(\d+)$").unwrap();
    let change_re = Regex::new(r"^c\s*$").unwrap();

    /* Match the input with the regular expressions */
    if quit_re.is_match(input) {
        BedCommand::Quit
    } else if print_re.is_match(input) {
        // start, end p  => print the lines from start to end
        let captures = print_re.captures(input).unwrap();

        let start = captures.get(1).map(|m| m.as_str().parse().unwrap());
        let end = captures.get(3).map(|m| m.as_str().parse().unwrap());

        let range = match input.contains(",") {
            true => {
                // if the second capture group is Some ','
                Range {
                    start: start.unwrap_or_else(|| 1),
                    end: end.unwrap_or_else(|| max_line),
                }
            }
            false => {
                // if the second capture group is None
                Range {
                    start: start.unwrap_or_else(|| current_line),
                    end: start.unwrap_or_else(|| current_line),
                }
            }
        };

        if input.ends_with("p") {
            BedCommand::Print { range }
        } else if input.ends_with("n") {
            BedCommand::NPrint { range }
        } else {
            eprintln!("Unknown command: {}", input);
            BedCommand::None
        }
    } else if change_re.is_match(input) {
        BedCommand::Change
    } else if move_re.is_match(input) {
        let line = move_re.captures(input).unwrap().get(1).unwrap().as_str().parse().unwrap();
        BedCommand::Move { line }
    } else {
        eprintln!("Unknown command: {}", input);
        BedCommand::None
    }
}

fn main() {
    /*
    Get the two first arguments from the command line

    - The first argument is the name of the program
    - The second argument is the name of the file to edit
    */
    let args: Vec<String> = env::args().collect();

    // if command line arguments are less than 2, print an error message
    if args.len() == 1 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    /* Open the file */
    let file = std::fs::read_to_string(&args[1]).unwrap();

    // Create the initial state of the editor
    let mut state = BedState {
        content: Rope::from(file),
        current_line: 0,
    };
    state.current_line = state.content.line_len();

    // Syntax highlighting
    let ps = SyntaxSet::load_defaults_newlines();

    // REPL loop
    loop {
        // print the prompt
        print!(":");
        std::io::stdout().flush().unwrap();

        // wait for the user to enter a command
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // get the file extension from the file name
        let ext = args[1].split('.').last().unwrap_or_else(|| "txt");
        let ts = ThemeSet::load_defaults();

        // create a hardcoded theme
        let theme_settings = ThemeSettings {
            foreground: Some(hex2color!("#a8dadc")), // Soft teal for text
            background: Some(Color::BLACK), // Deep black background
            caret: Some(hex2color!("#457B9D")), // Muted blue for caret
            line_highlight: Some(hex2color!("#1D3557")), // Subtle navy blue for active line
            misspelling: Some(hex2color!("#E63946")), // Warm red for misspellings
            minimap_border: Some(hex2color!("#2A2A2A")), // Subtle dark gray for minimap border
            accent: Some(hex2color!("#F4A261")), // Soft orange accent
            popup_css: Some("background-color: #2E2E2E; color: #A8DADC;".to_string()), // Softer contrast for popups
            phantom_css: Some("background-color: #3E3E3E; color: #E63946; border: 1px solid #F4A261;".to_string()),
            bracket_contents_foreground: Some(hex2color!("#F4A261")), // Soft orange for bracket contents
            bracket_contents_options: None,
            brackets_foreground: Some(hex2color!("#A8DADC")), // Matches foreground
            brackets_background: Some(hex2color!("#1D3557")), // Matches line highlight
            brackets_options: None,
            tags_foreground: Some(hex2color!("#F4A261")), // Soft orange for tags
            tags_options: None,
            highlight: Some(hex2color!("#2A2A2A")), // Subtle dark gray for highlights
            find_highlight: Some(hex2color!("#F4A261")), // Soft orange for find results
            find_highlight_foreground: Some(hex2color!("#1D3557")), // Navy text on orange
            gutter: Some(hex2color!("#1D3557")), // Matches line highlight
            gutter_foreground: Some(hex2color!("#A8DADC")), // Matches foreground
            selection: Some(hex2color!("#3E3E3E")), // Muted gray for selections
            selection_foreground: Some(hex2color!("#F1FAEE")), // Light cream text for selected content
            selection_border: Some(hex2color!("#F4A261")), // Soft orange border for selections
            inactive_selection: Some(hex2color!("#2E2E2E")), // Dark gray for inactive selections
            inactive_selection_foreground: Some(hex2color!("#A8DADC")), // Soft teal for inactive selection text
            guide: Some(hex2color!("#3E3E3E")), // Subtle gray guides
            active_guide: Some(hex2color!("#457B9D")), // Muted blue for active guide
            stack_guide: Some(hex2color!("#4E4E4E")), // Slightly brighter gray for stack guides
            shadow: Some(hex2color!("#000000")), // Pure black for shadows
        };


        let theme = Theme {
            name: Some(String::from("bed - theme")),
            author: Some(String::from("Luc Videau")),
            settings: theme_settings,
            scopes: ts.themes["base16-ocean.dark"].scopes.clone(),
        };

        let syntax = ps.find_syntax_by_extension(ext).unwrap();
        let mut h = HighlightLines::new(syntax, &theme);

        // parse the command
        let command = parse_command(&input, state.current_line, state.content.line_len());
        match command {
            BedCommand::None => continue,
            BedCommand::Quit => break,
            BedCommand::Change => {
                let mut new_content = String::new();
                std::io::stdin().read_line(&mut new_content).unwrap();
                state.content = Rope::from(new_content);
            }
            BedCommand::Print { range } => {
                for line in (range.start - 1)..range.end {
                    let string = state.content.line(line).to_string();
                    let ranges = h.highlight_line(&string, &ps).unwrap();
                    let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], true);
                    print!("{}\n", escaped);
                }
                print!("\x1b[0m");
            }
            BedCommand::NPrint { range } => {
                // get the width of the line number (the number of digits)
                let width = state.content.line_len().to_string().len();

                for line in (range.start - 1)..range.end {
                    let string = state.content.line(line).to_string();
                    let ranges = h.highlight_line(&string, &ps).unwrap();
                    let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], true);
                    // reset the color to the default color
                    print!("{:width$} │ ", line + 1, width = width);
                    print!("{}\n", escaped);
                    print!("\x1b[0m");
                }
            }
            BedCommand::Move { line } => {
                state.current_line = line;
            }
        }
    }
}
