use std::{collections::HashMap, cmp};
use std::io::{Read, Write};
use std::path::Path;
use std::fs::File;
use crossterm::event::{self, Event, KeyCode};
use tui::Terminal;
use tui::backend::Backend;
use std::{time::Duration};

use serde::{Deserialize, Serialize};

use crate::{cli, App, ui};
use cli::Cli;



#[derive(Serialize, Deserialize)]
struct ModeState{
    #[serde(default)]
    total_rounds: i32,
    #[serde(default)]
    games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub answer: String,
    pub guesses: Vec<String>,
}

pub struct Player {
    pub win_rounds: i32,
    pub total_rounds: i32,
    pub win_guess_times: Vec<i32>,
    pub hot_words: HashMap<String, i32>,
    pub games: Vec<Game>,
}

impl Player {
    pub fn new() -> Player {
        Player { 
            win_rounds: (0), 
            total_rounds: (0), 
            win_guess_times: (Vec::new()), 
            hot_words: (HashMap::new()), 
            games: (Vec::new()),
        } 
    }

    pub fn read_state_before(&mut self, cli: &Cli) -> Result<(), Box<dyn std::error::Error>>{
        match &cli.state {
            Some(file_path) => {
                if !cli.random {
                    return Err("--state/-S is only valid in random mode!".into());
                }
                let path = Path::new(file_path);
                if !path.exists() {
                    return Ok(());
                }
                let mut file = File::open(file_path)?;

                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                // read mode state
                let state_before: ModeState = serde_json::from_str(&contents)?;

                self.total_rounds = state_before.total_rounds;
                self.games = state_before.games;
                for game in &self.games {
                    for guess in &game.guesses {
                        let word_count = self.hot_words.entry(guess.clone()).or_insert(0);
                        *word_count += 1;
                    }
                    match game.guesses.last() {
                        Some(guess_last) => if &game.answer == guess_last {
                            self.win_rounds += 1;
                            self.win_guess_times.push(game.guesses.len() as i32);
                        }
                        None => (),
                    }
                }
                return Ok(());
            }
            None => Ok(()),
        }
    }

    pub fn write_state_after(&self, cli: &Cli) -> Result<(), Box<dyn std::error::Error>>{
        match &cli.state {
            Some(file_path) => {
                if !cli.random {
                    return Err("--state/-S only valid in random mode!".into());
                }
                let mut file = File::create(file_path)?;
                let state_after: ModeState = ModeState { total_rounds: (self.total_rounds), games: (self.games.clone()) };
                let contents = serde_json::to_string(&state_after)?;
                file.write(contents.as_bytes())?;
                return Ok(());
            }
            None => Ok(()),
        }
    }

    pub fn average_times(&self) -> f64{
        if self.win_rounds == 0 {
            return 0.0;
        }
        let mut win_guess_times_total = 0;
        for times in &self.win_guess_times {
            win_guess_times_total += times
        }
        let average_times = win_guess_times_total as f64 / self.win_rounds as f64;
        (average_times * 100.0).round() / 100.0
    }

    pub fn get_sorted_hot_words(&self) -> Vec<(String, i32)> {
        let mut hot_words_vec: Vec<(String, i32)> = Vec::new();
        for (key,value) in &self.hot_words {
            hot_words_vec.push((key.to_string(), *value));
        }

        // first string last i32
        hot_words_vec.sort_by(|a, b| a.0.cmp(&b.0));  
        hot_words_vec.sort_by(|a, b| b.1.cmp(&a.1));
        

        hot_words_vec
    }

    
    /// determine if there is a next game
    pub fn have_next_game<B: Backend>(&self, cli: &Cli, terminal:&mut Terminal<B>, app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(_word) = &cli.word {
            return Ok(false);
        }
        // statistics
        if cli.stats {
            let x = self.win_rounds;
            let y = self.total_rounds - x;
            let z = self.average_times(); 
            app.message += "\nwin rounds: ";
            app.message += x.to_string().as_str();
            app.message += ", lose rounds: ";
            app.message += y.to_string().as_str();
            app.message += ", average times: ";
            app.message += z.to_string().as_str();

            let sorted_hot_words = self.get_sorted_hot_words(); 
            // hot words:
            app.message += "\nHot words: ";
            let print_length = cmp::min(sorted_hot_words.len(), 5);
            for index in 0..print_length {
                
                app.message += sorted_hot_words[index].0.as_str(); 
                app.message += " ";
                app.message += sorted_hot_words[index].1.to_string().as_str();
                app.message += " ";

            }
            
        }
        app.message += "\nNext game: Y/N ...30 seconds before next round";
        terminal.draw(|f| ui(f, app))?;
        // write json

        // next game ?
        if crossterm::event::poll(Duration::from_secs(30))?{
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(ch) => {
                        match ch {
                            'y' => return Ok(true),
                            'Y' => return Ok(true),
                            _ => return Ok(false),
                        }
                    }
                    _ => return Ok(false),
                }
            }
        }
        return  Ok(true);

    }

}

