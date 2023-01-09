// use backgammon_cli::backgammon::Game;

// fn main() {
//     std::env::set_var("RUST_BACKTRACE", "1");
//     let mut game = Game::new();
//     game.start_vs_random();
// }

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style,
    style::{style, Attribute, Color, Stylize},
    terminal::{self, ClearType},
    Result,
};

use std::io::{self, Write};

fn read_char() -> Result<(char, KeyModifiers)> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: m,
            ..
        })) = event::read()
        {
            return Ok((c, m));
        }
    }
}

// // ANSI Shadow: ASCII Text
// const TITLE_SMALL_WIDTH: u16 = 90;
// const TITLE_SMALL: &str = r#"
// ██████╗  █████╗  ██████╗██╗  ██╗ ██████╗  █████╗ ███╗   ███╗███╗   ███╗ ██████╗ ███╗   ██╗
// ██╔══██╗██╔══██╗██╔════╝██║ ██╔╝██╔════╝ ██╔══██╗████╗ ████║████╗ ████║██╔═══██╗████╗  ██║
// ██████╔╝███████║██║     █████╔╝ ██║  ███╗███████║██╔████╔██║██╔████╔██║██║   ██║██╔██╗ ██║
// ██╔══██╗██╔══██║██║     ██╔═██╗ ██║   ██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██║   ██║██║╚██╗██║
// ██████╔╝██║  ██║╚██████╗██║  ██╗╚██████╔╝██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║╚██████╔╝██║ ╚████║
// ╚═════╝ ╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
// "#;

// Delta Corps Priest 1: ASCII Text
const TITLE_BIG_WIDTH: u16 = 123;
const TITLE_BIG_HEIGHT: u16 = 9;
const TITLE_BIG: &str = "\
▀█████████▄    ▄████████  ▄████████   ▄█   ▄█▄   ▄██████▄    ▄████████   ▄▄▄▄███▄▄▄▄     ▄▄▄▄███▄▄▄▄    ▄██████▄  ███▄▄▄▄ 
  ███    ███  ███    ███ ███    ███  ███ ▄███▀  ███    ███  ███    ███ ▄██▀▀▀███▀▀▀██▄ ▄██▀▀▀███▀▀▀██▄ ███    ███ ███▀▀▀██▄
  ███    ███  ███    ███ ███    █▀   ███▐██▀    ███    █▀   ███    ███ ███   ███   ███ ███   ███   ███ ███    ███ ███   ███
 ▄███▄▄▄██▀   ███    ███ ███        ▄█████▀    ▄███         ███    ███ ███   ███   ███ ███   ███   ███ ███    ███ ███   ███
▀▀███▀▀▀██▄ ▀███████████ ███       ▀▀█████▄   ▀▀███ ████▄ ▀███████████ ███   ███   ███ ███   ███   ███ ███    ███ ███   ███
  ███    ██▄  ███    ███ ███    █▄   ███▐██▄    ███    ███  ███    ███ ███   ███   ███ ███   ███   ███ ███    ███ ███   ███
  ███    ███  ███    ███ ███    ███  ███ ▀███▄  ███    ███  ███    ███ ███   ███   ███ ███   ███   ███ ███    ███ ███   ███
▄█████████▀   ███    █▀  ████████▀   ███   ▀█▀  ████████▀   ███    █▀   ▀█   ███   █▀   ▀█   ███   █▀   ▀██████▀   ▀█   █▀
                                     ▀";

const MENU_WIDTH: u16 = 34;
const MENU_HEIGHT: u16 = 5;
const MENU: &str = "\
[1] New Game (Player vs. Player)

[2] New Game (Player vs. Computer)

[q] Quit Game";

const TITLE_MENU_SEP: u16 = 5;
const TITLE_MENU_HEIGHT: u16 = TITLE_BIG_HEIGHT + TITLE_MENU_SEP + MENU_HEIGHT;

const PIECE: &str = "\
 ▄████▄ 
████████
 ▀████▀ ";

const POINT: &str = "
   ⣿
   ⣿
  ⢀⣿⡀
  ⢸⣿⡇
  ⢸⣿⡇
  ⣿⣿⣿
 ⢀⣿⣿⣿⡀
 ⢸⣿⣿⣿⡇
 ⢸⣿⣿⣿⡇
 ⣿⣿⣿⣿⣿
⢀⣿⣿⣿⣿⣿⡀
⢸⣿⣿⣿⣿⣿⡇
⢸⣿⣿⣿⣿⣿⡇
⣿⣿⣿⣿⣿⣿⣿
⣿⣿⣿⣿⣿⣿⣿";

//  ▄▄▄
// █████
// ▀███▀

const PIECES: [&str; 6] = [
    "
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀",
    "



 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀",
    "
╭─────╮
│ ╭─╮ │
│ ╰─╯ │
╰─────╯



 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀",
    "









 ▄▄▄
█████
▀███▀
 ▄▄▄
█████
▀███▀",
    "












 ▄▄▄
█████
▀███▀",
    "














",
];

fn main() -> Result<()> {
    #[cfg(windows)]
    let _ = enable_ansi_support();

    let mut stdout = io::stdout();

    let (cols, rows) = terminal::size()?;

    if cols < 128 {
        let error = style("Terminal window must be at least 128 columns wide.")
            .with(Color::Red)
            .attribute(Attribute::Bold);
        println!("{error}");
        return Ok(());
    }

    execute!(stdout, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    loop {
        queue!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;

        // for i in 0..6 {
        //     let color = if i % 2 == 0 {
        //         Color::White
        //     } else {
        //         Color::DarkGrey
        //     };
        //     for line in POINT.split('\n') {
        //         queue!(
        //             stdout,
        //             cursor::MoveRight(1 + i * 8),
        //             style::PrintStyledContent(style(line).with(color)),
        //             cursor::MoveToNextLine(1),
        //         )?;
        //     }

        //     queue!(stdout, cursor::MoveTo(0, 0))?;
        // }

        // queue!(stdout, cursor::MoveTo(0, 0))?;

        // for i in 0..6 {
        //     let color = if i % 2 == 1 {
        //         Color::White
        //     } else {
        //         Color::DarkGrey
        //     };
        //     for line in PIECES[i as usize].split('\n') {
        //         queue!(
        //             stdout,
        //             cursor::MoveRight(2 + i * 8),
        //             style::PrintStyledContent(style(line).with(color)),
        //             cursor::MoveToNextLine(1),
        //         )?;
        //     }

        //     queue!(stdout, cursor::MoveTo(0, 0))?;
        // }

        queue!(
            stdout,
            cursor::MoveToNextLine((rows - TITLE_MENU_HEIGHT) / 5 * 2)
        )?;

        for line in TITLE_BIG.split('\n') {
            queue!(
                stdout,
                cursor::MoveRight((cols - TITLE_BIG_WIDTH) / 2),
                style::Print(line),
                cursor::MoveToNextLine(1),
            )?;
        }

        queue!(stdout, cursor::MoveToNextLine(TITLE_MENU_SEP))?;

        for line in MENU.split('\n') {
            queue!(
                stdout,
                cursor::MoveRight((cols - MENU_WIDTH) / 2),
                style::Print(line),
                cursor::MoveToNextLine(1),
            )?;
        }

        stdout.flush()?;

        match read_char()? {
            ('1', _) => {}
            ('q', _) | ('c', KeyModifiers::CONTROL) => break,
            _ => {}
        }
    }

    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()?;

    Ok(())
}

//  ▄████▄
// ████████
//  ▀████▀
//  ▄████▄
// ████████
//  ▀████▀
//  ▄████▄
// ████████
//  ▀████▀
//  ▄████▄
// ████████
//  ▀████▀
//  ▄████▄   ▄████▄   ▄████▄   ▄████▄   ▄████▄   ▄████▄            ▄████▄   ▄████▄   ▄████▄   ▄████▄   ▄████▄   ▄████▄
// ████████ ████████ ████████ ████████ ████████ ████████          ████████ ████████ ████████ ████████ ████████ ████████
//  ▀████▀   ▀████▀   ▀████▀   ▀████▀   ▀████▀   ▀████▀            ▀████▀   ▀████▀   ▀████▀   ▀████▀   ▀████▀   ▀████▀
// ▄██▄
// ▀██▀
// ▄██▄
// ▀██▀
// ▄██▄
// ▀██▀
// ▄██▄
// ▀██▀
// ⡇⢸⣿⡆⢰⡄⢠⢀

//    ⣿
//    ⣿
//   ⢀⣿⡀
//   ⢸⣿⡇
//   ⢸⣿⡇
//   ⣿⣿⣿
//  ⢀⣿⣿⣿⡀
//  ⢸⣿⣿⣿⡇
//  ⢸⣿⣿⣿⡇
//  ⣿⣿⣿⣿⣿
// ⢀⣿⣿⣿⣿⣿⡀
// ⢸⣿⣿⣿⣿⣿⡇
// ⢸⣿⣿⣿⣿⣿⡇
// ⣿⣿⣿⣿⣿⣿⣿
// ⣿⣿⣿⣿⣿⣿⣿
// ████████

//   ⢸⡇
//   ⢸⡇
//   ⣼⣧
//   ⣿⣿
//   ⣿⣿
//  ⢸⣿⣿⡇
//  ⢸⣿⣿⡇
//  ⣼⣿⣿⣧
//  ⣿⣿⣿⣿
//  ⣿⣿⣿⣿
// ⢸⣿⣿⣿⣿⡇
// ⢸⣿⣿⣿⣿⡇
// ⣼⣿⣿⣿⣿⣧
// ⣿⣿⣿⣿⣿⣿
// ⣿⣿⣿⣿⣿⣿
╔═╗
╚═╝