use tui::{
    style::{Color, Modifier},
    widgets::BorderType,
};

pub struct Theme {
    pub active_row_input_color: Color,
    pub border_color: Color,
    pub header_text_error_color: Color,
    pub header_text_success_color: Color,
    pub empty_row_block_color: Color,
    pub guess_in_right_place_color: Color,
    pub guess_in_word_color: Color,
    pub guess_not_in_word_color: Color,
    pub keyboard_not_guessed_color: Color,
    pub keyboard_in_right_place_color: Color,
    pub keyboard_in_word_color: Color,
    pub keyboard_not_in_word_color: Color,
    pub row_border_thickness: BorderType,
    pub guessed_row_border_thickness: BorderType,
    pub welcome_message_color: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    pub fn light_theme() -> Self {
        Self {
            border_color: Color::Black,
            active_row_input_color: Color::Black,
            welcome_message_color: Color::Black,
            header_text_success_color: Color::Green,
            header_text_error_color: Color::Red,
            empty_row_block_color: Color::Gray,
            guess_in_right_place_color: Color::Green,
            guess_in_word_color: Color::Yellow,
            guess_not_in_word_color: Color::Black,
            keyboard_not_guessed_color: Color::Black,
            keyboard_in_right_place_color: Color::Green,
            keyboard_in_word_color: Color::Yellow,
            keyboard_not_in_word_color: Color::Gray,
            row_border_thickness: BorderType::Plain,
            guessed_row_border_thickness: BorderType::Thick,
        }
    }

    pub fn dark_theme() -> Self {
        Theme {
            border_color: Color::White,
            active_row_input_color: Color::White,
            welcome_message_color: Color::White,
            keyboard_not_guessed_color: Color::White,
            keyboard_not_in_word_color: Color::Gray,
            ..Theme::light_theme()
        }
    }
}

pub struct BlockTheme {
    pub border_brightness: Modifier,
    pub border_color: Color,
    pub border_thickness: BorderType,
    pub text_color: Color,
}
