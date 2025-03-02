use crop::Rope;
use regex::Regex;
use std::string::ToString;
use std::{env, io::Write};

/**
 * bed - a basic editor like ed but with a modern interface
 * Author: Luc Videau
 */

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
    Write,
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
    let write_re = Regex::new(r"^w\s*$").unwrap();

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
        let line = move_re
            .captures(input)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()
            .unwrap();
        BedCommand::Move { line }
    } else if write_re.is_match(input) {
        BedCommand::Write
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
        current_line: 1,
    };
    state.current_line = state.content.line_len();

    // REPL loop
    loop {
        // print the prompt
        print!(":");
        std::io::stdout().flush().unwrap();

        // wait for the user to enter a command
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        // execute the command
        let command = parse_command(&input, state.current_line, state.content.line_len());
        match command {
            BedCommand::None => continue,
            BedCommand::Quit => break,
            BedCommand::Write => {
                std::fs::write(&args[1], state.content.to_string()).unwrap();
            }
            BedCommand::Change => {
                // Get lines until regex ^.$ is matched
                let end_re = Regex::new(r"^\.\n$").unwrap();
                let mut new_content = String::new();
                loop {
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                    if end_re.is_match(&line) {
                        break;
                    }
                    new_content.push_str(&line);
                }
                // remove the current line from the content
                let byte_start = state.content.byte_of_line(state.current_line - 1);
                let byte_width = state.content.line(state.current_line - 1).byte_len();
                let byte_end = byte_start + byte_width;
                state.content.delete(byte_start..byte_end);
                // insert the new content at the current line
                state.content.insert(byte_start, &new_content.trim_end());
            }
            BedCommand::Print { range } => {
                for line in (range.start - 1)..range.end {
                    let string = state.content.line(line).to_string();
                    print!("{}\n", string);
                }
                print!("\x1b[0m");
            }
            BedCommand::NPrint { range } => {
                // get the width of the line number (the number of digits)
                let width = state.content.line_len().to_string().len();

                for line in (range.start - 1)..range.end {
                    let string = state.content.line(line).to_string();
                    // reset the color to the default color
                    print!("{:width$} │ ", line + 1, width = width);
                    print!("{}\n", string);
                    print!("\x1b[0m");
                }
            }
            BedCommand::Move { line } => {
                state.current_line = line;
            }
        }
    }
}
