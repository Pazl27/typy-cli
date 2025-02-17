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
            // not able to press space at the start of a line
            if game.player.position_x == 0 {
                return Ok(InputAction::Continue);
            }
            // check if is at end of line
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
            if game
                .get_word_string(game.player.position_y)
                .chars()
                .nth((game.player.position_x - 1) as usize)
                .context("Failed to get character from word")?
                == ' '
            {
                return Ok(InputAction::Continue);
            }
            if game.jump_position + 1 == game.player.position_x && game.jump_position != 0 {
                return Ok(InputAction::Continue);
            }
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
        }
        // check the typed letter
        if game.player.position_x
            < game.get_word_string(game.player.position_y).chars().count() as i32
        {
            if c == game
                .get_word_string(game.player.position_y)
                .chars()
                .nth(game.player.position_x as usize)
                .context("Failed to get character from word")?
            {
                stdout
                    .execute(SetForegroundColor(theme.fg))
                    .context("Failed to set foreground color")?;
                stdout
                    .execute(MoveTo(
                        x + game.player.position_x as u16,
                        y + game.player.position_y as u16,
                    ))
                    .context("Failed to move cursor")?;
                print!(
                    "{}",
                    game.get_word_string(game.player.position_y)
                        .chars()
                        .nth(game.player.position_x as usize)
                        .context("Failed to get character from word")?
                );
                stats.letter_count += 1;
            } else {
                stats.incorrect_letters += 1;
                stdout
                    .execute(SetForegroundColor(theme.error))
                    .context("Failed to set foreground color")?;
                stdout
                    .execute(MoveTo(
                        x + game.player.position_x as u16,
                        y + game.player.position_y as u16,
                    ))
                    .context("Failed to move cursor")?;
                print!(
                    "{}",
                    game.get_word_string(game.player.position_y)
                        .chars()
                        .nth(game.player.position_x as usize)
                        .context("Failed to get character from word")?
                );
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
        }
        stdout.flush().context("Failed to flush stdout")?;
    }
    Ok(InputAction::None)
}
