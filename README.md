# Wordlet

Wordlet is a command line clone of
[Wordle](https://www.powerlanguage.co.uk/wordle/), written in Rust.

![gameplay](https://user-images.githubusercontent.com/3178471/150548930-9dab1e11-2997-48da-af33-6e3386017a50.gif)

## Installation

`$ cargo install wordlet`

## Usage

After installation, start Wordlet by typing `wordlet` in your terminal. Unlike
Wordle, you can play at any time, and you can play multiple times per day. The
game uses the same dictionary as Wordle.

Valid options are:

- `--difficulty`, default is "easy". Can also be "hard".
- `--theme`, default is "dark". Can also be "light"

You quit the game by pressing escape.

## Nerd stuff

Building the app locally requires Rust 1.58 or higher.

This was an exercise in writing a fully functional Rust program. There are
probably better and more performant ways to implement the Wordlet algorithm but
I purposely did not look at how Wordle was implemented.

I learned a lot (and lifted some code) from these other games:
[Minesweeper](https://github.com/cpcloud/minesweep-rs) and
[Battleship](https://github.com/deepu105/battleship-rs). Both of those links
came from the [tui documentation](https://github.com/fdehau/tui-rs).

## Etc

- No rights reserved; use this code for whatever.
- I have no connection to Wordle or its author.
- [My twitter profile is here](https://twitter.com/scottluptowski)
