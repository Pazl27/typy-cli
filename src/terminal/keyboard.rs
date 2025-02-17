use std::io::Write;

use anyhow::{Context, Result};
use crossterm::cursor::MoveTo;
use crossterm::event::KeyCode;
use crossterm::style::SetForegroundColor;
use crossterm::ExecutableCommand;

use crate::{config::theme::ThemeColors, scores::stats::Stats};

use super::Game;

pub enum InputAction {
    Continue,
    Break,
    None,
}

pub fn handle_input(
    game: &mut Game,
    mut stdout: &std::io::Stdout,
    code: KeyCode,
    stats: &mut Stats,
    theme: &ThemeColors,
    x: u16,
    y: u16,
) -> Result<InputAction> {
    if let KeyCode::Char(c) = code {
        if c == ' ' {
            match handle_space(game, stdout, x, y)? {
                InputAction::Continue => return Ok(InputAction::Continue),
                InputAction::Break => return Ok(InputAction::Break),
                InputAction::None => {}
            };
        }
        // check the typed letter
        if game.player.position_x
            < game.get_word_string(game.player.position_y).chars().count() as i32
        {
            handle_chars(game, stats, theme, stdout, c, x, y)?;
        }
        stdout.flush().context("Failed to flush stdout")?;
    }
    Ok(InputAction::None)
}

fn handle_space(game: &mut Game, stdout: &std::io::Stdout, x: u16, y: u16) -> Result<InputAction> {
    if let InputAction::Continue = handle_start_of_line(game)? {
        return Ok(InputAction::Continue);
    }

    if let InputAction::Continue = handle_end_of_line(game, stdout, x, y)? {
        return Ok(InputAction::Continue);
    }

    if let InputAction::Continue = handle_space_in_word(game)? {
        return Ok(InputAction::Continue);
    }

    if game.jump_position + 1 == game.player.position_x && game.jump_position != 0 {
        return Ok(InputAction::Continue);
    }

    handle_jump_position(game, stdout, x, y)?;

    Ok(InputAction::None)
}

fn handle_start_of_line(game: &Game) -> Result<InputAction> {
    if game.player.position_x == 0 {
        return Ok(InputAction::Continue);
    }
    Ok(InputAction::None)
}

fn handle_end_of_line(
    game: &mut Game,
    mut stdout: &std::io::Stdout,
    x: u16,
    y: u16,
) -> Result<InputAction> {
    if game.selected_word_index
        == game
            .list
            .get(game.player.position_y as usize)
            .context("Failed to get word from list")?
            .len() as i32
            - 1
    {
        if game.player.position_y == game.list.len() as i32 {
            return Ok(InputAction::Break);
        }

        game.player.position_x = 0;
        game.player.position_y += 1;
        game.jump_position = 1;
        game.selected_word_index = 0;

        stdout
            .execute(MoveTo(
                x + game.player.position_x as u16,
                y + game.player.position_y as u16,
            ))
            .context("Failed to move cursor")?;
        return Ok(InputAction::Continue);
    }
    Ok(InputAction::None)
}

fn handle_space_in_word(game: &Game) -> Result<InputAction> {
    if game
        .get_word_string(game.player.position_y)
        .chars()
        .nth((game.player.position_x - 1) as usize)
        .context("Failed to get character from word")?
        == ' '
    {
        return Ok(InputAction::Continue);
    }
    Ok(InputAction::None)
}

fn handle_jump_position(
    game: &mut Game,
    mut stdout: &std::io::Stdout,
    x: u16,
    y: u16,
) -> Result<()> {
    game.jump_position = game
        .list
        .get(game.player.position_y as usize)
        .context("Failed to get word from list")?
        .iter()
        .take(game.selected_word_index as usize + 1)
        .map(|word| word.chars().count() + 1)
        .sum::<usize>() as i32
        - 1;
    game.player.position_x = game.jump_position;
    stdout
        .execute(MoveTo(
            x + game.player.position_x as u16,
            y + game.player.position_y as u16,
        ))
        .context("Failed to move cursor")?;
    game.selected_word_index += 1;
    Ok(())
}

fn handle_chars(
    game: &mut Game,
    stats: &mut Stats,
    theme: &ThemeColors,
    stdout: &std::io::Stdout,
    c: char,
    x: u16,
    y: u16,
) -> Result<()> {
    let expected_char = game
        .get_word_string(game.player.position_y)
        .chars()
        .nth(game.player.position_x as usize)
        .context("Failed to get character from word")?;

    if c == expected_char {
        handle_correct_char(game, theme, stdout, c, x, y)?;
    } else {
        handle_incorrect_char(game, theme, stdout, expected_char, x, y)?;
    }

    update_game_state(game, stats, c)?;

    Ok(())
}

fn handle_correct_char(
    game: &Game,
    theme: &ThemeColors,
    mut stdout: &std::io::Stdout,
    c: char,
    x: u16,
    y: u16,
) -> Result<()> {
    stdout
        .execute(SetForegroundColor(theme.fg))
        .context("Failed to set foreground color")?;
    stdout
        .execute(MoveTo(
            x + game.player.position_x as u16,
            y + game.player.position_y as u16,
        ))
        .context("Failed to move cursor")?;
    print!("{}", c);
    Ok(())
}

fn handle_incorrect_char(
    game: &Game,
    theme: &ThemeColors,
    mut stdout: &std::io::Stdout,
    c: char,
    x: u16,
    y: u16,
) -> Result<()> {
    stdout
        .execute(SetForegroundColor(theme.error))
        .context("Failed to set foreground color")?;
    stdout
        .execute(MoveTo(
            x + game.player.position_x as u16,
            y + game.player.position_y as u16,
        ))
        .context("Failed to move cursor")?;
    print!("{}", c);
    Ok(())
}

fn update_game_state(game: &mut Game, stats: &mut Stats, c: char) -> Result<()> {
    if c == game
        .get_word_string(game.player.position_y)
        .chars()
        .nth(game.player.position_x as usize)
        .context("Failed to get character from word")?
    {
        stats.letter_count += 1;
    } else {
        stats.incorrect_letters += 1;
        stats.letter_count += 1;
    }

    if game
        .get_word_string(game.player.position_y)
        .chars()
        .nth(game.player.position_x as usize)
        .context("Failed to get character from word")?
        == ' '
        && c != ' '
    {
        game.selected_word_index += 1;
    }
    game.player.position_x += 1;

    Ok(())
}
