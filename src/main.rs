use clap::Parser;
use std::{
    io::stdout,
    path::PathBuf,
};
use ui::UserInterface;

use crate::copy::CopyQueue;

pub mod copy;
pub mod ui;

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
    let queue = CopyQueue::from((PathBuf::from("test_dir"), vec![]));

    let ui = UserInterface::new().with_pre_copy(queue);
    ui.render(&mut stdout()).expect("Failed to render UI");
}
