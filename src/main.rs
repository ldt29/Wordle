use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal, 
};
use std::{io, time::Duration};
use clap::Parser;
use std::{collections::{HashMap, HashSet}};

mod builtin_words;

mod player;
use player::{Player, Game};
mod cli;
use cli::Cli;
mod server;
use server::Server;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // main logic
    main_logic(&mut terminal)?;

    // come back terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub struct App {
    guess_words: Vec<String>,
    word_states: Vec<Vec<u8>>,
    message: String,
    alphabet_state: Vec<u8>,
}
impl App {
    fn new() -> App {
        App { 
            guess_words: (Vec::new()), 
            word_states: (Vec::new()),
            message: ("Welcome to Wordle!\nPlease input word:".to_string()),
            alphabet_state: (vec!['X' as u8;26]),
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // area
    let chunks = Layout::default() // default
        .constraints([Constraint::Length(6), Constraint::Length(8), Constraint::Min(5)].as_ref()) // 按照 3 行 和 最小 3 行的规则分割区域
        .direction(Direction::Vertical) // vertical cutting
        .split(f.size()); // segment Terminal area

    // message
    let paragraph = Paragraph::new(Text::styled(
        app.message.to_string(),
        Style::default().add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("Wordle"))
    .alignment(tui::layout::Alignment::Left);

    f.render_widget(paragraph, chunks[0]);

    // input
    let mut input_text = Vec::new();
    for index in 0..app.guess_words.len() {
        let word_ascii = app.guess_words[index].clone().into_bytes();
        let mut word_char: Vec<char> = word_ascii.iter().map(|x| *x as char).collect();
        while word_char.len() < 5 {
            word_char.push('-');
            app.word_states[index].push('X' as u8);
        }

        let mut word_span = Vec::new();
        for letter in 0..5 {
            match app.word_states[index][letter] {
                71 => word_span.push(Span::styled(word_char[letter].to_string(), 
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                89 => word_span.push(Span::styled(word_char[letter].to_string(), 
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))),
                82 => word_span.push(Span::styled(word_char[letter].to_string(), 
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                _ => word_span.push(Span::styled(word_char[letter].to_string(),  
                    Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))),
            }
        }
        word_span.push("\n".into());
        input_text.push(Spans::from(word_span));
    }
    let paragraph = Paragraph::new(Text::from(input_text))
        .style(Style::default().bg(Color::White).fg(Color::Black))
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, chunks[1]);

    // keyboard
    let keyboard = vec!["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
    let mut keyboard_text = Vec::new();
    for index in 0..3 {
        let mut key_text =Vec::new();
        for ch in keyboard[index].as_bytes() {
            match app.alphabet_state[*ch as usize - 65] {
                71 => key_text.push(Span::styled((*ch as char).to_string(),
                  Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                89 => key_text.push(Span::styled((*ch as char).to_string(),
                  Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))),
                82 => key_text.push(Span::styled((*ch as char).to_string(),
                  Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                _ => key_text.push(Span::styled((*ch as char).to_string(), 
                  Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))),
            }
        }
        key_text.push("\n".into());
        keyboard_text.push(Spans::from(key_text));
    }
    let paragraph = Paragraph::new(Text::from(keyboard_text))
    .style(Style::default().bg(Color::White).fg(Color::Black))
    .block(Block::default().borders(Borders::ALL).title("Keyboard"))
    .alignment(Alignment::Center);

    f.render_widget(paragraph, chunks[2]);

}

/// The main logic function for the Wordle game, implement your own logic here
/// 
/// 
/// 
fn main_logic<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {

    let mut cli = Cli::parse();
    cli.mix_with_config()?;
    let mut player = Player::new();
    player.read_state_before(&cli)?;
    let mut server = Server::new(&cli);
    server.word_list_process(&cli)?;
    let mut app =App::new();
    terminal.draw(|f| ui(f, &mut app))?;
    loop {
        // process other logic
        server.init_secret_word(&cli, terminal, &mut app)?;
        play_game(&mut server, &mut player, &cli, terminal, &mut app)?;
        player.write_state_after(&cli)?;
        if player.have_next_game(&cli, terminal, &mut app)? == false {
            break;
        }
    }

    Ok(())
}




/// play game to guess secret word, we can try 6 times
fn play_game<B: Backend>(server: &mut Server, player: &mut Player, cli: &Cli, terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn std::error::Error>> 
{
    app.alphabet_state = vec!['X' as u8; 26];
    app.message = "Welcome to Wordle!\nRound ".to_string();
    app.message += (player.total_rounds + 1).to_string().as_str();
    app.message += "\nPlease input word:";
    app.guess_words.clear();
    app.word_states.clear();
    app.guess_words.push(String::new());
    app.word_states.push(Vec::new());
    terminal.draw(|f| ui(f, app))?;
    app.word_states.pop();
    app.guess_words.pop();

    let mut guess_count = 0;
    let mut word_states: Vec<Vec<u8>> = Vec::new();
    let mut guess_words: Vec<String> = Vec::new();
    let mut last_word_state: Vec<u8> = Vec::new();
    let mut last_guess_word: String = String::new();
    let mut alphabet_state: Vec<u8> = vec!['X' as u8; 26];
    player.total_rounds += 1;
    //server.recommend_n_possible_answers(&word_states, &guess_words, &cli.prompt);
    while guess_count < 6 {


        let mut guess_word = String::new();
        let mut word_state: Vec<u8> = Vec::new();
        // process keyboard input
        // block
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
                        if guess_word.len() < 5 {
                            guess_word.push(ch);
                            word_state.push(88);
                            app.guess_words.push(guess_word.clone());
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
                        guess_word.pop();
                        word_state.pop();
                        app.guess_words.push(guess_word.clone());
                        app.word_states.push(word_state.clone());
                        terminal.draw(|f| ui(f, app))?;
                        app.guess_words.pop();
                        app.word_states.pop();
                    }
                    _ => {}
                }
            }
        }

        guess_word = guess_word.trim().to_string();
        guess_word.make_ascii_uppercase();

        if server.is_valid(&guess_word, &last_word_state, &last_guess_word, cli.difficult) {

            guess_count += 1;
            guess_words.push(guess_word.clone());

            let word_count = player.hot_words.entry(guess_word.clone()).or_insert(0);
            *word_count += 1;

            let mut word_state: Vec<u8> = vec!['R' as u8; 5];

            let is_exit: bool = compare_secret_guess(&server.answer, &guess_word, &mut word_state, &mut alphabet_state); 

            word_states.push(word_state.clone());

            app.alphabet_state = alphabet_state.clone();
            app.guess_words.push(guess_word.clone());
            app.guess_words.push(String::new());
            app.word_states.push(word_state.clone());
            app.word_states.push(Vec::new());
            app.message = "Word is Wrong\nPlease input word again:".to_string();
            terminal.draw(|f| ui(f, app))?;
            

            if is_exit {
                app.word_states.pop();
                app.guess_words.pop();
                // if guess == secret, exit 
                app.message = "CORRECT with times: ".to_string() + &guess_count.to_string();
                terminal.draw(|f| ui(f, app))?;
                // statistics
                player.win_rounds +=1;
                player.win_guess_times.push(guess_count);
                player.games.push(Game{answer: (server.answer.clone()), guesses: (guess_words)});
                return Ok(());
            }
            last_guess_word = guess_word;
            last_word_state = word_state;

            server.recommend_n_possible_answers(&word_states, &guess_words, &cli.prompt, terminal, app)?;
            app.word_states.pop();
            app.guess_words.pop();

        }else {
            app.message = "Word is invalid\nPlease input word again:".to_string();
            app.word_states.push(Vec::new());
            app.guess_words.push(String::new());
            terminal.draw(|f| ui(f, app))?;

            server.recommend_n_possible_answers(&word_states, &guess_words, &cli.prompt, terminal, app)?;
            app.word_states.pop();
            app.guess_words.pop();
        }
    }
    
    // failed!!!
    app.message = "FAILED and answer is ".to_string() + &server.answer;
    terminal.draw(|f| ui(f, app))?;
    player.games.push(Game{answer: (server.answer.clone()), guesses: (guess_words)});
    Ok(())

}




pub fn compare_secret_guess(secret_word: &String, guess_word: &String, word_state: &mut Vec<u8>, alphabet_state: &mut Vec<u8>) -> bool {
    *word_state = get_word_state(secret_word, guess_word);
    let mut count_equal = 0;
    let guess_word_assci: Vec<u8> = guess_word.clone().into_bytes();


    for index in 0..5 {
        let guess_letter = guess_word_assci[index];
        match word_state[index] {
            71 => {
                count_equal += 1;
                
                let index_alphabet = guess_letter as usize - 65;
                alphabet_state[index_alphabet] = 'G' as u8;
            }
            89 => {
                let index_alphabet = guess_letter as usize - 65;
                if alphabet_state[index_alphabet] != 'G' as u8 {
                    alphabet_state[index_alphabet] = 'Y' as u8;
                }
            }
            82 => {
                let index_alphabet = guess_letter as usize - 65;
                if alphabet_state[index_alphabet] != 'G' as u8 && alphabet_state[index_alphabet] != 'Y' as u8 {
                    alphabet_state[index_alphabet] = 'R' as u8;
                }
            }
            _ => ()
        }
    }
    
    count_equal == 5

}


pub fn get_word_state(secret_word: &String, guess_word: &String) -> Vec<u8> {

    let mut word_state: Vec<u8> = vec!['R' as u8; 5];

    // remember secret index to guess index 
    let mut secret_guess: HashMap<usize, usize> = HashMap::new();
    // remeber guess letter index
    let mut guess_letter_maped: HashSet<usize> = HashSet::new();

    let secret_word_assci: Vec<u8> = secret_word.clone().into_bytes();
    let guess_word_assci: Vec<u8> = guess_word.clone().into_bytes();

    for index in 0..5 {
        let guess_letter = guess_word_assci[index];
        let secret_letter = secret_word_assci[index];

        if guess_letter == secret_letter {
            word_state[index] = 'G' as u8; 
            secret_guess.insert(index, index);
            guess_letter_maped.insert(index);
        }
    }

    for index in 0..5 {

        if guess_letter_maped.contains(&index) {
            continue;
        }

        let guess_letter = guess_word_assci[index];

        let check_secret = vec_search(&secret_word_assci, guess_letter);
        match check_secret {
            Some(index_of_secrets) => {
                for index_of_secret in index_of_secrets {
                    let check_guess = secret_guess.get(&index_of_secret);
                    if let None = check_guess {
                        word_state[index] = 'Y' as u8;
                        secret_guess.insert(index_of_secret, index);
                        break;
                    }
                }
            }
            None => (),
        }
    }

    word_state

}






// search all secret_letter that can map guess_letter
fn vec_search(secret_word_assci: &Vec<u8>, guess_letter: u8) -> Option<Vec<usize>> {
    let len = secret_word_assci.len();
    let mut all_index: Vec<usize> = Vec::new();
    for index in 0..len {
        if secret_word_assci[index] == guess_letter {
            all_index.push(index);
        }
    }
    if all_index.is_empty() {
        None
    }else {
        Some(all_index)
    }
}

/*
01000001	65	41	A	 
01000010	66	42	B	 
01000011	67	43	C	 
01000100	68	44	D	 
01000101	69	45	E	 
01000110	70	46	F	 
01000111	71	47	G	 
01001000	72	48	H	 
01001001	73	49	I	 
01001010	74	4A	J	 
01001011	75	4B	K	 
01001100	76	4C	L	 
01001101	77	4D	M	 
01001110	78	4E	N	 
01001111	79	4F	O	 
01010000	80	50	P	 
01010001	81	51	Q	 
01010010	82	52	R	 
01010011	83	53	S	 
01010100	84	54	T	 
01010101	85	55	U	 
01010110	86	56	V	 
01010111	87	57	W	 
01011000	88	58	X	 
01011001	89	59	Y	 
01011010	90	5A	Z 
*/