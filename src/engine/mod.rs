use crate::engine::game_error::GameError;

use std::collections::{HashMap, HashSet};

mod game_error;
mod utils;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameStatus {
    Won,
    InProgress,
    Lost,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GuessResult {
    DoesNotIncludeRequiredLetter(char),
    LetterDoesNotMatch(char, usize),
    DuplicateGuess,
    GameIsAlreadyOver,
    IncorrectCharacterCount,
    NotInDictionary,
    Valid,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum HitAccuracy {
    InRightPlace,
    InWord,
    NotInWord,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameDifficulty {
    Easy,
    Hard,
}

pub struct Game {
    guesses: Vec<WordGuess>,
    answer: String,
    difficulty: GameDifficulty,
    game_status: GameStatus,
    correct_positions: HashSet<usize>,
    dictionary: HashSet<String>,
    played_letters: HashMap<char, HitAccuracy>,
    row_states: Vec<RowState>,
}

#[derive(Debug, PartialEq)]
pub struct WordGuess {
    pub letters: Vec<GuessLetter>,
}

impl WordGuess {
    pub fn word(&self) -> String {
        self.letters
            .as_slice()
            .into_iter()
            .map(|gl| gl.letter)
            .collect()
    }

    pub fn letters(&self) -> &[GuessLetter] {
        self.letters.as_slice()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GuessLetter {
    pub letter: char,
    pub accuracy: HitAccuracy,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RowState {
    Empty,
    Current,
    AlreadyGuessed,
}

pub struct GameOptions {
    pub answer: Option<String>,
    pub difficulty: GameDifficulty,
}

impl Default for GameOptions {
    fn default() -> Self {
        GameOptions {
            answer: None,
            difficulty: GameDifficulty::Easy,
        }
    }
}

impl Game {
    pub fn new(args: GameOptions) -> Self {
        Game {
            guesses: Vec::with_capacity(6),
            answer: args
                .answer
                .map_or_else(|| utils::get_random_word(), |a| a.to_string()),
            difficulty: args.difficulty,
            game_status: GameStatus::InProgress,
            correct_positions: HashSet::new(),
            dictionary: utils::dictionary(),
            played_letters: HashMap::new(),
            row_states: vec![
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
            ],
        }
    }

    pub fn game_status(&self) -> GameStatus {
        self.game_status
    }

    pub fn get_answer(&self) -> Result<String, GameError> {
        if self.game_status == GameStatus::Lost {
            Ok(self.answer.to_string())
        } else {
            Err(GameError::GameNotLostError)
        }
    }

    pub fn guesses(&self) -> &[WordGuess] {
        self.guesses.as_slice()
    }

    fn in_dictionary(&self, word: &str) -> bool {
        self.dictionary.get(word).is_some()
    }

    fn answer_char_at_index(&self, index: usize) -> char {
        self.answer.chars().nth(index).unwrap()
    }

    fn matches_answer_at_index(&self, index: usize, letter: char) -> bool {
        letter == self.answer_char_at_index(index)
    }

    fn recalculate_row_states(&mut self) -> () {
        let number_of_guesses_so_far = self.guesses().len();

        let row_states = vec![1, 2, 3, 4, 5, 6]
            .into_iter()
            .map(|i| {
                if number_of_guesses_so_far == 6 {
                    return RowState::AlreadyGuessed;
                }

                if i <= number_of_guesses_so_far {
                    return RowState::AlreadyGuessed;
                }

                if i == number_of_guesses_so_far + 1 {
                    if self.game_status == GameStatus::Won {
                        return RowState::Empty;
                    }
                    return RowState::Current;
                }

                RowState::Empty
            })
            .collect();

        self.row_states = row_states;
        ()
    }

    fn recalculate_played_letter_registry(&mut self, guess: &WordGuess) -> () {
        for gl in guess.letters() {
            match self.played_letters.get_mut(&gl.letter) {
                None => {
                    self.played_letters.insert(gl.letter, gl.accuracy);
                }
                Some(accuracy_value) => {
                    if gl.accuracy < *accuracy_value {
                        *accuracy_value = gl.accuracy;
                    }
                }
            }
        }
    }

    fn guess_already_exists(&self, guess_input: &str) -> bool {
        let existing_guesses: Vec<String> = self.guesses.iter().map(|g| g.word()).collect();
        existing_guesses.contains(&guess_input.to_string())
    }

    pub fn guess(&mut self, guess_input: &str) -> (GameStatus, GuessResult) {
        if self.game_status == GameStatus::Won || self.game_status == GameStatus::Lost {
            return (self.game_status, GuessResult::GameIsAlreadyOver);
        }

        if guess_input.len() != 5 {
            return (self.game_status, GuessResult::IncorrectCharacterCount);
        }

        if self.guess_already_exists(&guess_input) {
            return (self.game_status, GuessResult::DuplicateGuess);
        }

        if !self.in_dictionary(&guess_input) {
            return (self.game_status, GuessResult::NotInDictionary);
        }

        if self.difficulty == GameDifficulty::Hard {
            for (index, letter) in guess_input.chars().enumerate() {
                if self.correct_positions.contains(&index) {
                    if !self.matches_answer_at_index(index, letter) {
                        let char_at_index = self.answer_char_at_index(index);
                        return (
                            self.game_status,
                            // we start counting at 1, so we can say "the first letter"
                            GuessResult::LetterDoesNotMatch(char_at_index, index + 1),
                        );
                    }
                }
            }

            for letter in self.answer.chars() {
                let is_discovered = self.is_letter_uncovered(letter);

                if is_discovered {
                    if !guess_input.contains(letter) {
                        return (
                            self.game_status,
                            GuessResult::DoesNotIncludeRequiredLetter(letter),
                        );
                    }
                }
            }
        }

        let guess = self.build_guess(&guess_input);
        self.recalculate_played_letter_registry(&guess);

        self.guesses.push(guess);

        if guess_input == self.answer {
            self.game_status = GameStatus::Won;
        }

        // we need to do this _after setting the game state to 'won', but before returning
        // This way the board does not update with a duplicate row in the next 'current' row
        self.recalculate_row_states();

        if self.game_status == GameStatus::Won {
            return (self.game_status, GuessResult::Valid);
        }

        if self.guesses.len() == 6 {
            self.game_status = GameStatus::Lost;
        }

        (self.game_status, GuessResult::Valid)
    }

    pub fn row_states(&self) -> Vec<RowState> {
        self.row_states.clone()
    }

    pub fn is_letter_uncovered(&self, letter: char) -> bool {
        match &self.get_letter_match_state(letter) {
            None => false,
            Some(HitAccuracy::NotInWord) => false,
            Some(_) => true,
        }
    }

    fn build_guess(&mut self, guess_input: &str) -> WordGuess {
        let mut discoverable_letters = utils::build_letter_counts(&self.answer);
        let mut guess_letters: Vec<Option<GuessLetter>> = vec![None, None, None, None, None];

        // Weird stuff. We walk the word twice; We go over the correct guesses first, so that we
        // can subtract their letters from the count of available letters to colorize.
        for (idx, c) in guess_input.chars().enumerate() {
            if self.matches_answer_at_index(idx, c) {
                guess_letters[idx] =
                    Some(self.build_guess_letter_with_accuracy(idx, c, &mut discoverable_letters))
            }
        }

        // Then we go over the letters that are not correct.
        for (idx, c) in guess_input.chars().enumerate() {
            if guess_letters[idx].is_none() {
                guess_letters[idx] =
                    Some(self.build_guess_letter_with_accuracy(idx, c, &mut discoverable_letters))
            }
        }

        WordGuess {
            letters: guess_letters.iter().map(|o| o.unwrap()).collect(),
        }
    }

    fn build_guess_letter_with_accuracy(
        &mut self,
        letter_index: usize,
        raw_letter: char,
        discoverable_letters: &mut HashMap<char, usize>,
    ) -> GuessLetter {
        let accuracy = match &self.answer.contains(raw_letter) {
            true => {
                let in_same_place = self.matches_answer_at_index(letter_index, raw_letter);

                if in_same_place {
                    if let Some(ch) = discoverable_letters.get_mut(&raw_letter) {
                        *ch -= 1;
                    }
                    self.correct_positions.insert(letter_index);
                    HitAccuracy::InRightPlace
                } else {
                    if let Some(ch) = discoverable_letters.get_mut(&raw_letter) {
                        if (*ch) >= 1 {
                            *ch -= 1;
                            HitAccuracy::InWord
                        } else {
                            HitAccuracy::NotInWord
                        }
                    } else {
                        HitAccuracy::NotInWord
                    }
                }
            }
            false => HitAccuracy::NotInWord,
        };

        GuessLetter {
            letter: raw_letter,
            accuracy: accuracy,
        }
    }

    pub fn get_letter_match_state(&self, letter: char) -> Option<HitAccuracy> {
        self.played_letters.get(&letter).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_guess() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("pasta");
        assert_eq!(game.guesses.len(), 1)
    }

    #[rustfmt::skip]
    #[test]
    fn test_a_guess_is_stored_correctly() {
        let mut game = Game::new(GameOptions { answer: Some("haste".to_string()), difficulty: GameDifficulty::Easy});
        game.guess("heart");

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 'h', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'a', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'r', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 't', accuracy: HitAccuracy::InWord }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[rustfmt::skip]
    #[test]
    fn test_letters_are_marked_in_word_until_the_count_of_letters_is_met() {
        let mut game = Game::new(GameOptions { answer: Some("sleep".to_string()), difficulty: GameDifficulty::Easy});
        game.guess("spell");
        // we guess spell. Only one of the l's should match as InWord, because there is only one l in sleep
        // Similarly, only one of the e's should match

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 's', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'p', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'l', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'l', accuracy: HitAccuracy::NotInWord }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[rustfmt::skip]
    #[test]
    fn test_counts_apply_to_the_in_right_place_characters_first() {
        let mut game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy});
        game.guess("added");
        // The guess 'added' has 3 'd' characters, but the answer only has one.
        // The 'd' char in the correct place (the last char) should be marked as in the right place,
        // and the other chars should be marked as NotInWord

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 'a', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::InRightPlace }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[test]
    fn test_cannot_add_duplicate_guess() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("pasta");
        let (_, duplicate_guess) = game.guess("pasta");
        assert_eq!(duplicate_guess, GuessResult::DuplicateGuess);
    }

    #[test]
    fn test_a_correct_guess_wins_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (won_the_game, _) = game.guess("slump");
        assert_eq!(won_the_game, GameStatus::Won);
    }

    #[test]
    fn test_a_guess_cannot_be_less_than_five_characters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (_, char_count_wrong) = game.guess("slp");
        assert_eq!(char_count_wrong, GuessResult::IncorrectCharacterCount);
    }

    #[test]
    fn test_a_guess_cannot_be_more_than_five_characters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (_, char_count_wrong) = game.guess("slumffffp");
        assert_eq!(char_count_wrong, GuessResult::IncorrectCharacterCount);
    }

    #[test]
    fn test_the_game_is_lost_after_six_incorrect_guesses() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit");
        game.guess("adorn");
        game.guess("adult");
        game.guess("affix");
        game.guess("afire");
        let (lost_the_game, _) = game.guess("after");
        assert_eq!(lost_the_game, GameStatus::Lost);
    }

    #[test]
    fn test_cannot_add_guesses_after_the_game_is_won() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("slump");
        let (won_the_game, game_already_over) = game.guess("adept");

        assert_eq!(won_the_game, GameStatus::Won);
        assert_eq!(game_already_over, GuessResult::GameIsAlreadyOver);
    }

    #[test]
    fn test_cannot_add_guesses_after_the_game_is_lost() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit");
        game.guess("adorn");
        game.guess("adult");
        game.guess("affix");
        game.guess("afire");
        game.guess("aging");

        let (lost_the_game, game_already_over) = game.guess("agony");
        assert_eq!(lost_the_game, GameStatus::Lost);
        assert_eq!(game_already_over, GuessResult::GameIsAlreadyOver);
    }

    #[test]
    fn test_cannot_add_a_word_that_does_not_exist() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (game_continues, invalid_word) = game.guess("djkle");
        assert_eq!(game_continues, GameStatus::InProgress);
        assert_eq!(invalid_word, GuessResult::NotInDictionary);
    }

    #[test]
    fn test_can_get_the_answer_after_the_game_is_lost() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit");
        game.guess("adorn");
        game.guess("adult");
        game.guess("affix");
        game.guess("afire");

        assert_eq!(game.get_answer(), Err(GameError::GameNotLostError));
        game.guess("aging");
        assert_eq!(game.get_answer(), Ok("slump".to_string()));
    }

    #[test]
    fn test_hard_mode_requires_guessing_letters_that_have_been_found_in_place() {
        let mut game = Game::new(GameOptions {
            answer: Some("abbey".to_string()),
            difficulty: GameDifficulty::Hard,
        });
        game.guess("sleep");

        let (_, required_letter) = game.guess("hours");
        assert_eq!(required_letter, GuessResult::LetterDoesNotMatch('e', 4));
    }

    #[test]
    fn test_hard_mode_requires_guessing_letters_that_have_been_found_in_the_word() {
        let mut game = Game::new(GameOptions {
            answer: Some("abbey".to_string()),
            difficulty: GameDifficulty::Hard,
        });
        let (_, valid_word) = game.guess("slept");
        assert_eq!(valid_word, GuessResult::Valid);

        let (_, required_letter) = game.guess("grift");
        assert_eq!(
            required_letter,
            GuessResult::DoesNotIncludeRequiredLetter('e')
        );
    }

    #[test]
    fn test_hard_mode_can_include_guesses_with_old_and_new_letters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            difficulty: GameDifficulty::Hard,
        });
        game.guess("sleep");

        let (game_continues, valid_word) = game.guess("sloop");
        assert_eq!(game_continues, GameStatus::InProgress);
        assert_eq!(valid_word, GuessResult::Valid);
    }

    #[test]
    fn test_keeps_track_of_which_letters_matched() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("slept");

        assert_eq!(
            game.get_letter_match_state('s'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('l'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('e'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(game.get_letter_match_state('p'), Some(HitAccuracy::InWord));
        assert_eq!(
            game.get_letter_match_state('t'),
            Some(HitAccuracy::NotInWord)
        );

        assert_eq!(game.get_letter_match_state('u'), None);
        assert_eq!(game.get_letter_match_state('m'), None);
    }

    #[test]
    fn test_letters_matches_are_not_overwritten_by_lesser_tiers() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("larva");

        assert_eq!(
            game.get_letter_match_state('l'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('a'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('r'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(
            game.get_letter_match_state('v'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(
            game.get_letter_match_state('a'),
            Some(HitAccuracy::InRightPlace)
        );

        assert_eq!(game.get_letter_match_state('g'), None);
        assert_eq!(game.get_letter_match_state('h'), None);
    }

    #[test]
    fn test_letters_matches_are_not_overwritten_by_subsequent_incorrect_guesses() {
        let mut game = Game::new(GameOptions {
            answer: Some("ahead".to_string()),
            ..Default::default()
        });
        // we guess 'lease'. The first 'e' should match as InWord, and the second should be NotInWord
        // When we ask for the letter match state, it should respond with InWord
        game.guess("lease");
        assert_eq!(game.get_letter_match_state('e'), Some(HitAccuracy::InWord));

        // now we've guessed the correct letter, so it should correct to InRightPlace
        game.guess("preen");
        assert_eq!(
            game.get_letter_match_state('e'),
            Some(HitAccuracy::InRightPlace)
        );
    }

    #[test]
    fn test_row_states_are_tracked_at_the_start_of_the_game() {
        let game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        assert_eq!(
            game.row_states(),
            vec![
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_in_the_middle_of_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit");

        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_when_you_win_before_the_end() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit");
        game.guess("laugh");

        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_at_the_end_of_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit");
        game.guess("adorn");
        game.guess("adult");
        game.guess("affix");
        game.guess("afire");
        game.guess("aging");
        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed
            ]
        );
    }
}
