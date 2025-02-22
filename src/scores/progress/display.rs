use anyhow::Result;
use super::*;

pub fn draw() -> Result<()>{

    let mut scores = Score::get_scores()?;
    Score::sort_scores(&mut scores);

    todo!();

    Ok(())

}
