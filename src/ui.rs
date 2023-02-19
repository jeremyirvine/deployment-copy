use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    execute, queue,
    style::{Print, Stylize},
    terminal::{Clear, ClearType},
};
use std::{
    io::{stdout, Stdout, Write},
    sync::mpsc::Receiver,
};

use crate::copy::CopyQueue;

///
/// # Deployment Copy UI
///
/// ### Arrow Placement
/// With 1 entry, no decision is needed
/// With 2 entries we use the first item to point the arrow
///
/// ```
/// ╭────────────┬──────────────────────────────────────╮
/// │  test-dir ──>  .rustc_info.json                   │
/// │            │   CACHEDIR.TAG                       │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
/// ```
/// ╭────────────┬──────────────────────────────────────╮
/// │            │  .rustc_info.json                    │
/// │  test-dir ──> CACHEDIR.TAG                        │
/// │            │  debug                               │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
/// ```
/// ╭────────────┬──────────────────────────────────────╮
/// │            │  .rustc_info.json                    │
/// │  test-dir ──> CACHEDIR.TAG                        │
/// │            │  debug                               │
/// │            │  .fingerprint                        │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
///
///
/// ```
/// ╭───────────────────────────────────────────────────╮
/// │ Deployment Copy                                   │
/// ├───────────────────────────────────────────────────┤
/// │ Do you want to copy to these directories?         │
/// │ Press [Y] or [N] on your keyboard                 │
/// ╰───────────────────────────────────────────────────╯
/// ╭────────────┬──────────────────────────────────────╮
/// │            │  .rustc_info.json                    │
/// │  test-dir ──> CACHEDIR.TAG                        │
/// │            │  debug                               │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
///
/// ```
/// ╭───────────────────────────────────────────────────╮
/// │ Deployment Copy                                   │
/// ├───────────────────────────────────────────────────┤
/// │ Copying...                                    D:\ │
/// │ 10mb copied (20%)                                 │
/// ╰───────────────────────────────────────────────────╯
/// ╭────────────┬──────────────────────────────────────╮
/// │            │  .rustc_info.json                    │
/// │  test-dir ──> CACHEDIR.TAG                        │
/// │            │  debug                               │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
///
///
/// ```
/// ╭───────────────────────────────────────────────────╮
/// │ Deployment Copy                                   │
/// ├───────────────────────────────────────────────────┤
/// │ Finished Copying                                  │
/// │ 10mb copied (20%)                                 │
/// ╰───────────────────────────────────────────────────╯
/// ╭────────────┬──────────────────────────────────────╮
/// │            │  .rustc_info.json                    │
/// │  test-dir ──> CACHEDIR.TAG                        │
/// │            │  debug                               │
/// ╰────────────┴──────────────────────────────────────╯
/// ```
///
///

// Straight Pieces
#[allow(dead_code)]
const VERTICAL_CHAR: char = '│';

#[allow(dead_code)]
const HORIZONTAL_CHAR: char = '─';

// Split Pieces
#[allow(dead_code)]
const SPLIT_RIGHT: char = '┤';

#[allow(dead_code)]
const SPLIT_LEFT: char = '├';

#[allow(dead_code)]
const SPLIT_ABOVE: char = '┬';

#[allow(dead_code)]
const SPLIT_BELOW: char = '┴';

// Corner Pieces (styled w/ rounded edges)
#[allow(dead_code)]
const BOTTOM_LEFT_CHAR: char = '╰';

#[allow(dead_code)]
const BOTTOM_RIGHT_CHAR: char = '╯';

#[allow(dead_code)]
const TOP_LEFT_CHAR: char = '╭';

#[allow(dead_code)]
const TOP_RIGHT_CHAR: char = '╮';

#[allow(dead_code)]
const BOX_WIDTH: usize = 51;

#[derive(Clone)]
pub struct CopyingState {
    pub mb_copied: usize,
    pub percentage: usize,
}

pub enum UIState {
    PreCopy(CopyQueue),
    Copying(Receiver<CopyingState>),
    Completed(CopyQueue),
}

pub struct UserInterface {
    state: Option<UIState>,
}

impl Default for UserInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl UserInterface {
    pub fn new() -> Self {
        execute!(stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
        Self { state: None }
    }

    pub fn with_pre_copy(mut self, queue: CopyQueue) -> Self {
        self.state = Some(UIState::PreCopy(queue));
        self
    }

    pub fn with_copying(mut self, channel: Receiver<CopyingState>) -> Self {
        self.state = Some(UIState::Copying(channel));
        self
    }

    pub fn with_completed(mut self, queue: CopyQueue) -> Self {
        self.state = Some(UIState::Completed(queue));
        self
    }

    pub fn render(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        match self.state {
            Some(UIState::PreCopy(ref queue)) => self.render_pre_copy(stdout, queue),
            _ => Ok(())
        }?;

        stdout.flush()
    }

    fn render_header(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        let title = "Deployment Copy";
        let width = BOX_WIDTH - (title.len() + 1);

        queue!(
            stdout,
            MoveTo(0, 0),
        )?;
        self.render_side_top(stdout)?;
        queue!(
            stdout,
            MoveToNextLine(1),
            Print(format!(
                "{} {}{: >width$}{}",
                VERTICAL_CHAR,
                title.magenta(),
                "",
                VERTICAL_CHAR,
            )),
            MoveToNextLine(1),
            Print(format!(
                "{}{}{}",
                SPLIT_LEFT,
                (HORIZONTAL_CHAR.to_string()).repeat(BOX_WIDTH),
                SPLIT_RIGHT,
            )),
            MoveToNextLine(1),
        )
    }

    fn render_lines(&self, stdout: &mut Stdout, content: Vec<impl Into<String>>) -> Result<(), std::io::Error> {
        for line in content {
            let line = line.into();
            let width = BOX_WIDTH
                .checked_sub(line.unformat().len())
                .unwrap_or(BOX_WIDTH) - 1;

            queue!(
                stdout,
                Print(format!(
                    "{} {}{: <width$}{}",
                    VERTICAL_CHAR, line, "", VERTICAL_CHAR,
                )),
                MoveToNextLine(1),
            )?;
        }
        Ok(())
    }

    fn render_side_top(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        queue!(
            stdout,
            Print(format!(
                "{}{}{}",
                TOP_LEFT_CHAR,
                (HORIZONTAL_CHAR.to_string()).repeat(BOX_WIDTH),
                TOP_RIGHT_CHAR,
            )),
        )
    }

    fn render_side_bottom(&self, stdout: &mut Stdout) -> Result<(), std::io::Error> {
        queue!(
            stdout,
            Print(format!(
                "{}{}{}",
                BOTTOM_LEFT_CHAR,
                (HORIZONTAL_CHAR.to_string()).repeat(BOX_WIDTH),
                BOTTOM_RIGHT_CHAR,
            )),
            MoveToNextLine(1),
        )
    }

    fn render_queue(&self, _stdout: &mut Stdout, queue: &CopyQueue) -> Result<(), std::io::Error> {
        // TODO: Implement queue render function
        let _arrow_index = match queue.destinations().len() {
            0 => None,
            1 | 2 => Some(1),
            c => Some(c/2),
        };
        Ok(())
    }

    fn render_pre_copy(&self, stdout: &mut Stdout, queue: &CopyQueue) -> Result<(), std::io::Error> {
        self.render_header(stdout)?;
        self.render_lines(stdout, vec![
            "Do you want to copy to these directories?".into(),
            format!(
                "Press {} or {} on your keyboard",
                "[Y]".dark_grey().bold(),
                "[N]".dark_grey().bold()
            ),
        ])?;
        self.render_side_bottom(stdout)?;
        self.render_queue(stdout, queue)
    }
}

// i know not strictly necessary, but writing this out every time i need to strip ANSI
// escape codes is annoying and makes the code a bit more unreadable
pub trait Unformattable {
    fn unformat(&self) -> Self;
}
impl Unformattable for String {
    fn unformat(&self) -> Self {
        String::from_utf8_lossy(&strip_ansi_escapes::strip(self).unwrap()).to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn can_create_pre_copy() {
        let queue = CopyQueue::from((PathBuf::from("D:\\"), vec![]));
        let ui = UserInterface::new().with_pre_copy(queue);

        match ui.state {
            Some(UIState::PreCopy(_)) => {}
            _ => panic!("Failed to create PreCopy UI"),
        }
    }

    #[test]
    fn can_create_copying() {
        let (_, rx) = ::std::sync::mpsc::channel::<CopyingState>();
        let ui = UserInterface::new().with_copying(rx);

        match ui.state {
            Some(UIState::Copying(_)) => {}
            _ => panic!("Failed to create Copying UI"),
        }
    }

    #[test]
    fn can_create_completed() {
        let queue = CopyQueue::from((PathBuf::from("D:\\"), vec![]));
        let ui = UserInterface::new().with_completed(queue);

        match ui.state {
            Some(UIState::Completed(_)) => {}
            _ => panic!("Failed to create Completed UI"),
        }
    }

    #[test]
    fn can_remove_styles_from_string() {
        let styled_string = format!("Hello, {}!", "World".red().bold());
        assert_eq!(styled_string.unformat(), String::from("Hello, World!"),)
    }
}
