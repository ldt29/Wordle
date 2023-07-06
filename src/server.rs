use rand::{rngs::StdRng, SeedableRng};
use rand::prelude::*;
use tui::backend::Backend;
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::{collections::HashSet};
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use crossterm::event::{self, Event, KeyCode};
use std::{io, time::Duration};
use crossterm::{
    event::{ DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tui::{
    backend::{ CrosstermBackend},
    Terminal, 
};

use crate::{builtin_words, get_word_state, App, ui};
use builtin_words::FINAL;
use builtin_words::ACCEPTABLE;

use crate::cli;
use cli::Cli;

pub struct Server{
    pub answer: String,
    final_words: Vec<String>,
    acceptable_words: Vec<String>,
    rounds: i32,
    pub possible_answer: Vec<String>,
    entropy_count: i32,
}

impl Server {
    pub fn new(cli: &Cli) -> Server {
        Server { 
            answer: (String::new()), 
            final_words: (Vec::new()),
            acceptable_words: (Vec::new()),
            rounds: (cli.day.unwrap()),
            possible_answer: (Vec::new()),
            entropy_count: (0),
        }
    }

    // procee final word list and acceptable word list
    pub fn word_list_process(&mut self, cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
        // check final-set
        match &cli.final_set {
            Some(file_path) => {
                let mut file = File::open(file_path)?;
                let mut final_contents = String::new();
                file.read_to_string(&mut final_contents)?;
                final_contents = final_contents.trim().to_string();
                self.final_words = final_contents.split("\n").map(|s| s.trim().to_uppercase()).collect();
                let final_word_set: HashSet<_> = self.final_words.iter().collect();
                
                if final_word_set.len() < self.final_words.len() {
                    return Err("final word list have same word!".into());
                }

                for word in &final_word_set {
                    if !word_basic_check(&word.to_string()){
                        return Err("error final word!".into());
                    }
                }
                
            }
            None => {
                self.final_words = FINAL.to_vec().iter().map(|s| s.trim().to_uppercase()).collect()
            }
        }
        // check acceptable-set
        match &cli.acceptable_set {
            Some(file_path) => {
                let mut file = File::open(file_path)?;
                let mut acceptable_contents = String::new();
                file.read_to_string(&mut acceptable_contents)?;
                acceptable_contents = acceptable_contents.trim().to_string();
                self.acceptable_words = acceptable_contents.split("\n").map(|s| s.trim().to_uppercase()).collect();
                let acceptable_word_set: HashSet<_> = self.final_words.iter().collect();
                
                if acceptable_word_set.len() < self.final_words.len() {
                    return Err("acceptable word list have same word!".into());
                }

                for word in &acceptable_word_set {
                    if !word_basic_check(&word.to_string()){
                        return Err("error acceptable word!".into());
                    }
                }
            }
            None => self.acceptable_words = ACCEPTABLE.to_vec().iter().map(|s| s.trim().to_uppercase()).collect(),
        }

        // subset
        let final_word_set: HashSet<_> = self.final_words.iter().collect();
        let acceptable_word_set: HashSet<_> = self.acceptable_words.iter().collect();
        if !final_word_set.is_subset(&acceptable_word_set) {
            return Err("final is not subset of acceptable!".into());
        }

        // sort by dirctionary list
        self.acceptable_words.sort_by(|a, b| a.cmp(b));    
        self.final_words.sort_by(|a, b| a.cmp(b)); 

        // shuffle
        let mut rng: StdRng = StdRng::seed_from_u64(cli.seed.unwrap());
        self.final_words.shuffle(&mut rng);

        Ok(())
    }

    /// initialize secret word
    pub fn init_secret_word<B: Backend>(&mut self, cli: &Cli, terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>>{
        if cli.random {
            self.answer = self.final_words[self.rounds as usize - 1].clone();
            self.rounds += 1;
        }
        else {
            match &cli.word{
                Some(word) => self.answer = word.to_string(),
                None => {
                    app.alphabet_state = vec!['X' as u8; 26];
                    app.message = "Welcome to Wordle!\nPlease input word for guess:".to_string();
                    app.guess_words.clear();
                    app.word_states.clear();
                    app.guess_words.push(String::new());
                    app.word_states.push(Vec::new());
                    terminal.draw(|f| ui(f, app))?;
                    app.word_states.pop();
                    app.guess_words.pop();
        
                    // Please input a word which has 5 bytes:
                    // process keyboard input
                    // block
                    let mut word = String::new();
                    let mut word_state: Vec<u8> = Vec::new();
                    while crossterm::event::poll(Duration::from_secs(60))?{
                        if let Event::Key(key) = event::read()? {
                            match key.code {
                                KeyCode::Esc => {
                                    // come back terminal
                                    let backend = CrosstermBackend::new(io::stdout());
                                    let mut terminal = Terminal::new(backend)?;
                                    disable_raw_mode()?;
                                    execute!(
                                        terminal.backend_mut(),
                                        LeaveAlternateScreen,
                                        DisableMouseCapture
                                    )?;
                                    terminal.show_cursor()?;
                                    return Err("Force Quit.".into());
                                }
                                KeyCode::Char(ch) => {
                                    if word.len() < 5 {
                                        word.push(ch);
                                        word_state.push(88);
                                        app.guess_words.push(word.clone());
                                        app.word_states.push(word_state.clone());
                                        terminal.draw(|f| ui(f, app))?;
                                        app.guess_words.pop();
                                        app.word_states.pop();
                                    }
                                }
                                KeyCode::Enter => {
                                    break;
                                }
                                KeyCode::Backspace => {
                                    word.pop();
                                    word_state.pop();
                                    app.guess_words.push(word.clone());
                                    app.word_states.push(word_state.clone());
                                    terminal.draw(|f| ui(f, app))?;
                                    app.guess_words.pop();
                                    app.word_states.pop();
                                }
                                _ => {}
                            }
                        }
                    }
                    self.answer = word.trim().to_string();
                }
            }
        }

        self.answer.make_ascii_uppercase();
        Ok(())
    }

    
    /// word is valid or not
    pub fn is_valid(&self, word: &String, last_word_state: &Vec<u8>, last_guess_word: &String, is_hard: bool) -> bool {

        if !word_basic_check(word) {
            return false;
        }


        // in hard mode, next guess word only depend last guess word :), markov process!!!
        if is_hard && !last_guess_word.is_empty() {
            if !word_hard_check(word, last_word_state, last_guess_word) {
                return false;
            }
        }

        for word_acceptable in &self.acceptable_words {
            if word_acceptable == word {
                return true;
            }
        }

        false
    }

    fn get_all_possible_answers(&mut self, word_states: &Vec<Vec<u8>>, guess_words: &Vec<String>) {
        self.possible_answer.clear();

        if guess_words.len() == 0 {
            self.possible_answer = self.acceptable_words.clone();
        }
        else {
            for acceptable_word in &self.acceptable_words {
                let mut all_meet: usize = 0;

                for i in 0..guess_words.len() {
                    if word_perfect_check(acceptable_word, &word_states[i], &guess_words[i]) {
                        all_meet += 1;
                    }
                }

                if all_meet == guess_words.len() {
                    self.possible_answer.push(acceptable_word.clone());
                }

            }
        }

    }

    pub fn recommend_n_possible_answers<B: Backend>(&mut self, word_states: &Vec<Vec<u8>>, guess_words: &Vec<String>, prompt: &Option<i32>, terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        match prompt {
            Some( n ) => {
                self.get_all_possible_answers(word_states, guess_words);
                
                let mut recommend_words_vec: Vec<(String, f64)> = Vec::new();
                
                self.entropy_count = 0;

                // rayon for speed!
                self.possible_answer
                    .par_iter()
                    .map(|x| (x.to_string(), self.compute_entropy_for_one(x)))
                    .collect_into_vec(&mut recommend_words_vec);


                recommend_words_vec.sort_by(|a, b| a.0.cmp(&b.0));  
                recommend_words_vec.sort_by(|a, b| OrderedFloat(b.1).cmp(&OrderedFloat(a.1)));

                app.message += "\nPossible answer and entropy:\n";
                let len = min(self.possible_answer.len(), *n as usize);
                 
                for i in 0..len {
                    app.message += recommend_words_vec[i as usize].0.as_str();
                    app.message += " ";
                    let mut temp = recommend_words_vec[i as usize].1;
                    temp = (temp * 100.0).round() / 100.0;
                    app.message += temp.to_string().as_str();
                    app.message += " ";
                }
                terminal.draw(|f| ui(f,app))?;
                Ok(())
            }
            None => Ok(()),
        }
    }
    

    fn compute_entropy_for_one(&self, word: &String) -> f64 {

        let mut all_match_count: Vec<i32> = vec![0; 243];
        let total_count = self.possible_answer.len() as f64; 
        let mut match_word_states: Vec<Vec<u8>> = Vec::new();

        self.possible_answer
            .par_iter()
            .map(|x| get_word_state(x, word))
            .collect_into_vec(&mut match_word_states);

        for word_state in &match_word_states {
            
            // word state => index
            let mut index: i32 = 0;
            for letter in word_state {
                match letter {
                    82 => index = index * 3,
                    89 => index = index * 3 + 1,
                    71 => index = index * 3 + 2,
                    _ => (),
                }
            }
            all_match_count[index as usize] += 1;
        }
        
        // filter 0
        all_match_count = all_match_count.into_par_iter().filter(|x| *x != 0).collect();

        // compute entropy with possiblity
        let state_possiblities: Vec<f64> = all_match_count.par_iter().map(|x| *x as f64 / total_count ).collect();
        let state_information: Vec<f64> = state_possiblities.par_iter().map(|x| - x * x.log(2.71828) ).collect();

        state_information.iter().sum()

    }

}


fn word_basic_check(word: &String) -> bool {
    if word.len() != 5 {
        return false;
    }
    let word_assci: Vec<u8> = word.clone().into_bytes();
    for word_a in &word_assci {
        if word_a < &65 || word_a > &90 {
            return false;
        }
    } 
    true
}

fn word_hard_check(word: &String, last_word_state: &Vec<u8>, last_guess_word: &String) -> bool {
    let word_assci: Vec<u8> = word.clone().into_bytes();
    let mut maped_index:HashSet<usize> = HashSet::new();
    let last_word_assci: Vec<u8> = last_guess_word.clone().into_bytes();

    for index in 0..5 { 
        if last_word_state[index] == 71 {
            if word_assci[index] != last_word_assci[index] {
                return false;
            }
            maped_index.insert(index);
        }
    } //G

    for index in 0..5 { 
        if last_word_state[index] == 89 {
            let mut is_false = true;
            for letter in 0..5 {
                if maped_index.contains(&letter) {
                    continue;
                }
                if word_assci[letter] == last_word_assci[index] {
                    maped_index.insert(letter);
                    is_false = false;
                    break;
                }
            }
            if is_false {
                return false;
            }
        }
    } //Y
    true
}

fn word_perfect_check(word: &String, last_word_state: &Vec<u8>, last_guess_word: &String) -> bool {
    let word_assci: Vec<u8> = word.clone().into_bytes();
    let last_word_assci: Vec<u8> = last_guess_word.clone().into_bytes();
    let mut maped_index:HashSet<usize> = HashSet::new();
    
    for index in 0..5 { 
        if last_word_state[index] == 71 {
            if word_assci[index] != last_word_assci[index] {
                return false;
            }
            maped_index.insert(index);
        }
    } //G

    for index in 0..5 { 
        if last_word_state[index] == 89 {
            if word_assci[index] == last_word_assci[index] {
                return false;
            }
            let mut is_false = true;
            for letter in 0..5 {
                if maped_index.contains(&letter) {
                    continue;
                }
                if word_assci[letter] == last_word_assci[index] {
                    maped_index.insert(letter);
                    is_false = false;
                    break;
                }
            }
            if is_false {
                return false;
            }
        }
    } //Y


    for index in 0..5 { 
        if last_word_state[index] == 82 {

            for letter in 0..5 {
                if maped_index.contains(&letter) {
                    continue;
                }
                if word_assci[letter] == last_word_assci[index] {
                    return false;
                    
                }
            }
        
        }
    } //R

    true
}