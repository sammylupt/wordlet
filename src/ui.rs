use crate::app::{App, Disclaimer};
use crate::engine::{GuessResult, HitAccuracy, RowState};
use crate::theme::BlockTheme;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub enum Error {
    ConvertUsizeToU16(std::num::TryFromIntError),
}

use Disclaimer::*;
use GuessResult::*;

const ROWS: usize = 6;
const COLUMNS: usize = 5;
const CELL_WIDTH: usize = 5;
const CELL_HEIGHT: usize = 3;
const PADDING: usize = 1;

pub fn draw<B: Backend>(frame: &mut Frame<B>, app: &mut App) -> Result<(), crate::ui::Error> {
    // a LOT of this code comes from a Minesweeper implementation in Rust, found at:
    // https://github.com/cpcloud/minesweep-rs/blob/main/src/ui.rs
    let terminal_rect = frame.size();
    let grid_width =
        u16::try_from(CELL_WIDTH * COLUMNS + 2 * PADDING).map_err(Error::ConvertUsizeToU16)?;
    let grid_height =
        u16::try_from(CELL_HEIGHT * ROWS + 2 * PADDING).map_err(Error::ConvertUsizeToU16)?;

    let row_constraints = std::iter::repeat(Constraint::Length(
        u16::try_from(CELL_HEIGHT).map_err(Error::ConvertUsizeToU16)?,
    ))
    .take(ROWS)
    .collect::<Vec<_>>();

    let col_constraints = std::iter::repeat(Constraint::Length(
        u16::try_from(CELL_WIDTH).map_err(Error::ConvertUsizeToU16)?,
    ))
    .take(COLUMNS)
    .collect::<Vec<_>>();

    let outer_rects = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(1)
        .horizontal_margin(1)
        .constraints(vec![Constraint::Min(grid_height)])
        .split(frame.size());

    let game_rectangle = outer_rects[0];

    let horizontal_pad_block_width = (terminal_rect.width - grid_width) / 2;
    let center_center_horizontally = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Min(horizontal_pad_block_width),
            Constraint::Length(grid_width),
            Constraint::Min(horizontal_pad_block_width),
        ])
        .split(game_rectangle);

    let vertical_pad_block_height = (game_rectangle.height - grid_height) / 2;
    let center_content_vertically = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(vertical_pad_block_height),
            Constraint::Length(grid_height),
            Constraint::Min(vertical_pad_block_height),
        ])
        .split(center_center_horizontally[1]);

    let top_section_render_thing = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(center_content_vertically[0]);

    let keyboard_render_things = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(center_content_vertically[2]);

    let game_board = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let game_board_section = center_content_vertically[1];
    frame.render_widget(game_board, game_board_section);
    draw_header(frame, app, top_section_render_thing[0]);
    draw_keyboard(frame, app, keyboard_render_things[1]);

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(1)
        .horizontal_margin(0)
        .constraints(row_constraints.clone())
        .split(game_board_section);

    let board_state = app.game.row_states();

    for (row_index, row_chunk) in row_chunks.into_iter().enumerate() {
        let row = board_state[row_index];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .vertical_margin(0)
            .horizontal_margin(1)
            .constraints(col_constraints.clone())
            .split(row_chunk);

        match row {
            RowState::Current => render_active_row(frame, app, chunks),
            RowState::Empty => render_empty_row(frame, app, chunks),
            RowState::AlreadyGuessed => render_already_guessed_row(frame, app, row_index, chunks),
        }
    }

    Ok(())
}

pub fn render_empty_row<B: Backend>(
    frame: &mut Frame<B>,
    app: &mut App,
    cell_chunks: Vec<Rect>,
) -> () {
    for cell_chunk in cell_chunks.into_iter() {
        let content = render_cell_with_text_and_colors(
            " ".to_string(),
            BlockTheme {
                border_color: app.theme.empty_row_block_color,
                text_color: app.theme.empty_row_block_color,
                border_thickness: app.theme.row_border_thickness,
                border_brightness: Modifier::empty(),
            },
        );

        frame.render_widget(content, cell_chunk);
    }
}

pub fn render_active_row<B: Backend>(
    frame: &mut Frame<B>,
    app: &mut App,
    cell_chunks: Vec<Rect>,
) -> () {
    let mut chars = app.input.chars();

    for cell_chunk in cell_chunks.into_iter() {
        let text = match chars.next() {
            Some(l) => l.to_string(),
            _ => " ".to_string(),
        };
        let content = render_cell_with_text_and_colors(
            text,
            BlockTheme {
                border_color: app.theme.border_color,
                text_color: app.theme.active_row_input_color,
                border_thickness: app.theme.row_border_thickness,
                border_brightness: Modifier::empty(),
            },
        );
        frame.render_widget(content, cell_chunk);
    }
}

pub fn render_already_guessed_row<B: Backend>(
    frame: &mut Frame<B>,
    app: &mut App,
    row_index: usize,
    chunks: Vec<Rect>,
) -> () {
    if let Some(word_guess) = app.game.guesses().get(row_index) {
        let items = chunks.iter().zip(word_guess.letters.iter());

        for (chunk, guess_letter) in items {
            let character = guess_letter.letter.to_string();
            let accuracy = guess_letter.accuracy;

            let color = match accuracy {
                HitAccuracy::InRightPlace => app.theme.guess_in_right_place_color,
                HitAccuracy::InWord => app.theme.guess_in_word_color,
                HitAccuracy::NotInWord => app.theme.guess_not_in_word_color,
            };

            let brightness = match accuracy {
                HitAccuracy::NotInWord => Modifier::DIM,
                _ => Modifier::empty(),
            };

            let content = render_cell_with_text_and_colors(
                character,
                BlockTheme {
                    border_color: color,
                    text_color: color,
                    border_thickness: app.theme.guessed_row_border_thickness,
                    border_brightness: brightness,
                },
            );

            frame.render_widget(content, *chunk);
        }
    }
}

pub fn render_cell_with_text_and_colors(
    text: String,
    block_theme: BlockTheme,
) -> Paragraph<'static> {
    let text = formatted_cell_text(text);

    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(block_theme.border_thickness)
                .border_style(Style::default().fg(block_theme.border_color))
                .style(
                    Style::default()
                        .add_modifier(block_theme.border_brightness)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(block_theme.text_color))
}

// This is taken directly from the minesweeper app
// https://github.com/cpcloud/minesweep-rs/blob/main/src/ui.rs
fn formatted_cell_text(text: String) -> String {
    let single_row_text = format!("{:^length$}", text, length = CELL_WIDTH - 2);
    let pad_line = " ".repeat(CELL_WIDTH);
    let num_pad_lines = CELL_HEIGHT - 3;

    std::iter::repeat(pad_line.clone())
        .take(num_pad_lines / 2)
        .chain(std::iter::once(single_row_text.clone()))
        .chain(std::iter::repeat(pad_line).take(num_pad_lines / 2))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn draw_header<B: Backend>(frame: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let text = match &app.disclaimer {
        Some(GameWonMessage) => String::from("Game is over! You win! Press any key to exit."),
        Some(GameOverMessage(answer)) => {
            format!("Game over! The answer was '{answer}'. Press any key to exit.")
        }
        Some(MoveFeedback(f)) => match f {
            DoesNotIncludeRequiredLetter(letter) => {
                format!("Does not include the required letter '{letter}'")
            }
            LetterDoesNotMatch(ch, idx) => {
                let number = match idx {
                    1 => "1st".to_string(),
                    2 => "2nd".to_string(),
                    3 => "3rd".to_string(),
                    _ => format!("{ch}th"),
                };
                format!("The {number} letter must be '{ch}'")
            }
            IncorrectCharacterCount => String::from("Your guess must be 5 characters long!"),
            NotInDictionary => String::from("Not a valid word!"),
            DuplicateGuess => String::from("You already guessed that!"),
            GameIsAlreadyOver => String::from("The game is already over!"),
            Valid => String::from(""),
        },
        Some(WelcomeMessage) => {
            String::from("Welcome to Wordlet. You have six tries to guess the answer. Good luck!")
        }
        None => String::from(""),
    };

    let header_text_color = match &app.disclaimer {
        Some(GameWonMessage) => app.theme.header_text_success_color,
        Some(WelcomeMessage) => app.theme.welcome_message_color,
        _ => app.theme.header_text_error_color,
    };

    let header_text = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(header_text_color))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.theme.border_color))
                .title("Wordlet")
                .border_type(BorderType::Plain),
        );

    frame.render_widget(header_text, chunk);
}

pub fn draw_keyboard<B: Backend>(frame: &mut Frame<B>, app: &mut App, chunk: Rect) {
    let keyboard_key_rows = vec!["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let keyboard_spans = keyboard_key_rows
        .iter()
        .fold(vec![], |mut acc, keyboard_row| {
            // when we draw the keyboard, we want a blank space after every character
            // except for the last character, so that we don't go off-center
            let letters: Vec<Span> = keyboard_row
                .chars()
                .into_iter()
                .enumerate()
                .map(|(letter_index, letter)| {
                    let use_offset = letter_index != keyboard_row.len() - 1;
                    keyboard_letter(&app, letter, use_offset)
                })
                .collect();

            acc.push(Spans::from(letters));
            acc
        });

    let keyboard_visualization = Paragraph::new(keyboard_spans)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.theme.border_color))
                .title("Available Letters")
                .border_type(BorderType::Plain),
        );

    frame.render_widget(keyboard_visualization, chunk);
}

pub fn keyboard_letter<'a>(app: &'a App, le: char, use_offset: bool) -> Span<'a> {
    use HitAccuracy::*;
    let key_state = app.game.get_letter_match_state(le);

    let color = match key_state {
        None => app.theme.keyboard_not_guessed_color,
        Some(InRightPlace) => app.theme.keyboard_in_right_place_color,
        Some(InWord) => app.theme.keyboard_in_word_color,
        Some(NotInWord) => app.theme.keyboard_not_in_word_color,
    };

    let display_modifier = match key_state {
        Some(NotInWord) => Modifier::DIM,
        _ => Modifier::empty(),
    };

    let key_string = match use_offset {
        true => format!("{le} "),
        false => le.to_string(),
    };

    Span::styled(
        key_string,
        Style::default().fg(color).add_modifier(display_modifier),
    )
}
