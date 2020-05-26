mod primitives;
mod briscola;
mod beccaccino;
mod utils;
mod telegram;
mod threading;
mod bot;
mod game_agent;

use primitives::Game;
use std::sync::mpsc;
use std::collections::HashMap;

// A game can last up to 10 minutes since the last action
static MAX_GAME_DURATION: u64 = 600;

fn main() {
    // Data storage
    // Association between players and their respective games
    let mut player_games: HashMap<telegram_bot_raw::types::refs::UserId, String> = HashMap::new();
    let mut game_channel: HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>> = HashMap::new();
    let mut game_last_played: HashMap<String, std::time::Instant> = HashMap::new();
    
    let mut playable_games: Vec<Box<dyn Game>> = Vec::new();
    // List of playable games
    playable_games.push(Box::from(briscola::Briscola::default()));
    playable_games.push(Box::from(beccaccino::Beccaccino::default()));

    println!("Starting CardGamesBot...");
    let mut client = telegram::Telegram::init();
    bot::main_bot_logic(&playable_games, &mut player_games, &mut game_channel, &mut game_last_played, &mut client)
}
