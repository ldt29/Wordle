use clap::Parser;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

/// the Cli struct is for command lines args
#[derive(Parser, Serialize, Deserialize)]
#[command(name = "Wordle")]
#[command(author = "ldt20 <ldt20@mails.tsinghua.edu.cn>")]
#[command(version = "1.0")]
#[command(about = "Attention is all you need", long_about = None)]
pub struct Cli {
    /// assigned word mode
    #[arg(short, long)]
    pub word: Option<String>,
    /// random mode
    #[arg(short, long)]
    pub random: bool,
    /// hard mode
    #[arg(short = 'D', long)]
    pub difficult: bool,
    /// statistics for all games
    #[arg(short = 't', long)]
    pub stats: bool,
    /// begin at nth game
    #[arg(short = 'd', long)]
    pub day: Option<i32>,
    /// random seed
    #[arg(short = 's', long)]
    pub seed: Option<u64>,
    /// final word txt
    #[arg(short = 'f', long = "final-set")]
    pub final_set: Option<String>,
    /// acceptable word txt
    #[arg(short = 'a', long = "acceptable-set")]
    pub acceptable_set: Option<String>,
    /// state json
    #[arg(short = 'S', long)]
    pub state: Option<String>,
    /// config json
    #[arg(short, long)]
    pub config: Option<String>,
    /// give n prompt word 
    #[arg(short, long)]
    pub prompt: Option<i32>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    /// assigned word mode
    word: Option<String>,
    /// random mode
    random: Option<bool>,
    /// hard mode
    difficult: Option<bool>,
    /// statistics for all games
    stats: Option<bool>,
    /// begin at nth game
    day: Option<i32>,
    /// random seed
    seed: Option<u64>,
    /// final word txt
    final_set: Option<String>,
    /// acceptable word txt
    acceptable_set: Option<String>,
    /// state json
    state: Option<String>,
    /// config json
    // config: Option<String>,
    /// give n prompt word 
    prompt: Option<i32>,
}

impl Cli{
    pub fn mix_with_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match  &self.config {
            Some(file_path) => {
                let path = Path::new(file_path);
                if !path.exists() {
                    return Err("Not found path".into());
                }

                let mut file = File::open(file_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let cli_config: Config = serde_json::from_str(&contents)?;
   
                match &self.word {
                    Some(_word) => (),
                    None => {
                        match &cli_config.word {
                            Some(word) => self.word = Some(word.to_string()),
                            None => (),
                        }
                    },
                }
                
                match &self.random {
                    true => (),
                    false => {
                        match &cli_config.random {
                            Some(random) => self.random = *random,
                            None => (),
                        }
                    },
                }
                // random mode and worde mod can't exist at the same time
                
                match &self.difficult {
                    true => (),
                    false => {
                        match &cli_config.difficult {
                            Some(difficult) => self.difficult = *difficult,
                            None => (),
                        }
                    },
                }

                match &self.stats {
                    true => (),
                    false => {
                        match &cli_config.stats {
                            Some(stats) => self.stats = *stats,
                            None => (),
                        }
                    },
                }
                

                match &self.day {
                    Some(_day) => (),
                    None => {
                        match &cli_config.day {
                            Some(day) => self.day = Some(*day),
                            None => (),
                        }
                    },
                }
                match &self.seed {
                    Some(_seed) => (),
                    None => {
                        match &cli_config.seed {
                            Some(seed) => self.seed = Some(*seed),
                            None => (),
                        }
                    },
                }
                match &self.final_set {
                    Some(_final_set) => (),
                    None => {
                        match &cli_config.final_set {
                            Some(final_set) => self.final_set = Some(final_set.to_string()),
                            None => (),
                        }
                    },
                }

                match &self.acceptable_set {
                    Some(_acceptable_set) => (),
                    None => {
                        match &cli_config.acceptable_set {
                            Some(acceptable_set) => self.acceptable_set = Some(acceptable_set.to_string()),
                            None => (),
                        }
                    },
                }

                match &self.state {
                    Some(_state) => (),
                    None => {
                        match &cli_config.state {
                            Some(state) => self.state = Some(state.to_string()),
                            None => (),
                        }
                    },
                }

                match &self.prompt {
                    Some(_prompt) => (),
                    None => {
                        match &cli_config.prompt {
                            Some(prompt) => self.prompt = Some(*prompt),
                            None => (),
                        }
                    },
                }

            }
            None => ()

        }
        // check all args which have conflict
        self.check_conflict()?;

        Ok(())
    }   

    /// check all args which have conflict
    fn check_conflict(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //In word mode, you con't use -d/--day or -s/--seed!
        match &self.word {
            Some(_word) => {
                if self.day !=None || self.seed != None {
                    return Err("In word mode, you con't use -d/--day or -s/--seed!".into());
                }
            }
            None => (),
        }

        // day and seed should have default value
        match &self.day {
            Some(_day) => (),
            None => self.day = Some(1),
        }
        match &self.seed {
            Some(_seed) => (),
            None => self.seed = Some(42),
        }

        //Random mode and word mode can't exist at the same time
        if self.random {
            if self.word.is_some() {
                return Err("Random mode and word mode can't exist at the same time!".into());
            }
        } 
        match &self.day {
            Some(day) => {
                if *day < 1 {
                    return Err("day less than 1".into());
                }
            },
            None => (),
        }
        Ok(())
    } 
    
}