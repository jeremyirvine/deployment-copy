use clap::Parser;
use crossterm::{
    cursor::{MoveToColumn, MoveUp},
    queue,
    style::{Color, Print, SetForegroundColor, Stylize},
    terminal::{Clear, ClearType},
};
use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use crate::copy::CopyQueue;

pub mod copy;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg()]
    pub copy_from: PathBuf,

    #[arg()]
    pub drives: Vec<PathBuf>,

    #[arg(long, short)]
    pub yes: bool,
}

fn main() {
    let args = Args::parse();

    let mut copy_from = ::std::env::current_dir().expect("Failed to get current directory");
    copy_from.push(args.copy_from.clone());

    let dir = ::std::fs::read_dir(&copy_from)
        .unwrap_or_else(|_| panic!("Could not open directory `{}`", copy_from.display()));

    let dir_list = dir
        .filter(|d| d.is_ok())
        .map(|d| match d {
            Ok(dir) => (dir.path(), dir.file_name().to_string_lossy().to_string()),
            Err(_) => {
                unreachable!();
            }
        })
        .collect::<Vec<(PathBuf, String)>>();

    print_pre_copy_status(&dir_list, &args);

    if !args.yes {
        print!(
            "Does everything look correct? (You can disable this prompt with the `-y` flag) (Y/n) "
        );
        ::std::io::stdout().flush().expect("Failed to flush stdout");
        let mut buffer = String::new();
        ::std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to read user input");

        match buffer.replace("\r\n", "").to_lowercase().as_str() {
            "y" | "yes" => {
                queue!(
                    stdout(),
                    MoveUp(1),
                    Clear(ClearType::CurrentLine),
                    MoveToColumn(0)
                )
                .unwrap();

                stdout().flush().unwrap();
            }
            _ => {
                println!("[decopy] Aborting copy...");
                ::std::process::exit(0);
            }
        }
    }

    let mut queue = CopyQueue::from(&args);
    handle_copying(&mut queue);
}

fn print_pre_copy_status(dir_list: &Vec<(PathBuf, String)>, args: &Args) {
    log("Destinations staged to be copied to:\n");
    for drive in args.drives.clone() {
        println!("  {}", drive.display().to_string().dark_grey());
    }
    log(format!("Copying from `{}`...\n", args.copy_from.display()));
    let (list, is_overflowing) = if dir_list.len() >= 5 {
        (&dir_list[..5], true)
    } else {
        (&dir_list[..], false)
    };

    for (_, display) in list {
        println!("  {}", display.clone().dark_grey());
    }
    if is_overflowing {
        println!("  ... +{} more ...", dir_list.len() - list.len());
    }
}

pub fn handle_copying(queue: &mut CopyQueue) {
    // execute!(stdout(), MoveToNextLine(1)).unwrap();

    let onpercentage = move |percent: usize, current_dir: PathBuf, bytes_copied: usize| {
        queue!(stdout(), Clear(ClearType::CurrentLine), MoveToColumn(0),).unwrap();
        log_queue(format!(
            "Copying... ({} %) [{} copied] --> {}",
            percent,
            get_bytes_string(bytes_copied),
            current_dir.display()
        ));

        stdout().flush().unwrap();
    };

    let oncomplete = move || {
        queue!(stdout(), Print("\n")).unwrap();
        log("Files finished copying");
    };

    queue.start_copy(Box::new(onpercentage), Box::new(oncomplete));
}

pub fn log_queue(msg: impl Into<String>) {
    queue!(
        stdout(),
        Print("["),
        SetForegroundColor(Color::Magenta),
        Print("decopy"),
        SetForegroundColor(Color::Reset),
        Print("] "),
        Print(msg.into()),
    )
    .unwrap();
}

pub fn log(msg: impl Into<String>) {
    log_queue(msg);
    stdout().flush().unwrap();
}

pub fn get_bytes_string(bytes: usize) -> String {
    match bytes {
        bytes if bytes >= 1024usize.pow(4) => {
            format!("{}tb", bytes / 1024usize.pow(4))
        }
        bytes if bytes >= 1024usize.pow(3) => {
            format!("{}gb", bytes / 1024usize.pow(3))
        }
        bytes if bytes >= 1024usize.pow(2) => {
            format!("{}mb", bytes / 1024usize.pow(2))
        }
        bytes if bytes >= 1024 => {
            format!("{}kb", bytes / 1024)
        }
        n => format!("{}b", n),
    }
}
