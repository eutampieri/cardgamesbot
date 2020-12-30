mod bot;
mod game_agent;
mod primitives;
mod telegram;
mod threading;
mod utils;

use cardgames::primitives::Game;
use std::collections::HashMap;
use std::sync::mpsc;

// A game can last up to 10 minutes since the last action
static MAX_GAME_DURATION: u64 = 600;

fn main() {
    // Data storage
    // Association between players and their respective games
    let mut player_games: HashMap<telegram_bot_raw::types::refs::UserId, String> = HashMap::new();
    let mut game_channel: HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>> =
        HashMap::new();
    let mut game_last_played: HashMap<String, std::time::Instant> = HashMap::new();

    let mut playable_games: Vec<Box<dyn Game>> = Vec::new();
    // List of playable games
    playable_games.push(Box::from(cardgames::games::briscola::Briscola::default()));
    playable_games.push(Box::from(
        cardgames::games::beccaccino::Beccaccino::default(),
    ));
    playable_games.push(Box::from(cardgames::games::scala40::Scala40::default()));

    println!("Starting CardGamesBot...");
    let mut client = telegram::Telegram::init();
    bot::main_bot_logic(
        &playable_games,
        &mut player_games,
        &mut game_channel,
        &mut game_last_played,
        &mut client,
    )
}
