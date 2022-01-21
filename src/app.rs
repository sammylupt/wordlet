use crate::engine::{Game, GameOptions, GameStatus, GuessResult};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(PartialEq)]
pub enum Disclaimer {
    MoveFeedback(GuessResult),
    GameWonMessage,
    GameOverMessage(String),
    WelcomeMessage,
}

pub struct App {
    pub game: Game,
    pub input: String,
    pub disclaimer: Option<Disclaimer>,
    pub should_quit: bool,
    pub theme: Theme,
}

pub struct AppOptions {
    pub theme: Theme,
    pub game_config: GameOptions,
}

impl App {
    pub fn new(args: AppOptions) -> Self {
        App {
            game: Game::new(args.game_config),
            input: String::from(""),
            disclaimer: Some(Disclaimer::WelcomeMessage),
            should_quit: false,
            theme: args.theme,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> () {
        if self.game.game_status() != GameStatus::InProgress {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Backspace => self.on_backspace(),
            KeyCode::Enter => self.on_enter_press(),
            KeyCode::Char(letter) => self.on_letter_entered(letter),
            _ => (),
        };
    }

    pub fn on_valid_word(&mut self) -> () {
        self.disclaimer = None;
        self.input = String::from("");
    }

    pub fn on_backspace(&mut self) -> () {
        let _ = self.input.pop();
        ()
    }

    pub fn on_letter_entered(&mut self, letter: char) -> () {
        if self.input.chars().count() <= 4 {
            self.input.push(letter);
        }
    }

    pub fn on_enter_press(&mut self) -> () {
        // clear the disclaimer the first time a word is played
        if self.disclaimer == Some(Disclaimer::WelcomeMessage) {
            self.disclaimer = None;
        }

        if &self.input.chars().count() != &5 {
            return ();
        }

        match self.game.guess(&self.input) {
            (GameStatus::Lost, _) => {
                if let Ok(answer) = self.game.get_answer() {
                    self.disclaimer = Some(Disclaimer::GameOverMessage(answer.to_string()));
                }
            }
            (GameStatus::Won, _) => {
                self.disclaimer = Some(Disclaimer::GameWonMessage);
            }
            (_, word_res) => match word_res {
                GuessResult::Valid => {
                    let _ = &self.on_valid_word();
                }
                result => {
                    self.disclaimer = Some(Disclaimer::MoveFeedback(result));
                }
            },
        }
    }
}
