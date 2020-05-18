mod primitives;
mod briscola;
mod beccaccino;
mod utils;
mod telegram;
mod threading;

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
    let mut cleanup_list: Vec<String> = Vec::new();
    
    let mut playable_games: Vec<Box<dyn Game>> = Vec::new();
    // List of playable games
    playable_games.push(Box::from(briscola::Briscola::default()));
    playable_games.push(Box::from(beccaccino::Beccaccino::default()));

    println!("Starting CardGamesBot...");
    let mut client = telegram::Telegram::init();
    loop {
        use telegram_bot_raw::types::update::UpdateKind;
        use telegram_bot_raw::types::message::MessageKind;
        for update in client.get_updates() {
            if let UpdateKind::Message(msg) = update.kind {
                if let MessageKind::Text{data, entities} = msg.kind {
                    drop(entities); // Silence the stupid warning and free some RAM
                    if data.contains("/start") {
                        let pieces: Vec<String> = data.split(" ").map(|x| x.to_owned()).collect();
                        if pieces.len() == 1 {
                            client.send_message(("Ciao! A che gioco vuoi giocare?", msg.from.id, &playable_games).into());
                        } else {
                            let game_id = pieces[1].clone();
                            client.send_message(("Provo ad aggiungerti alla partita...", msg.from.id).into());
                            // Check wether the user is already playing a game
                            if player_games.contains_key(&msg.from.id) {
                                // The user can't join two games at the same time
                                client.send_message(("Non puoi unirti a più partite contemporaneamente", msg.from.id).into())
                            } else {
                                player_games.insert(msg.from.id, game_id.clone());
                                if let Some(ch) = game_channel.get(&game_id) {
                                    ch.send(threading::ThreadMessage::AddPlayer(primitives::Player{id: msg.from.id.into(), name: utils::get_user_name(&msg.from.first_name, &msg.from.last_name)})).unwrap();
                                } else {
                                    client.send_message(("Gioco non trovato!", msg.from.id).into())
                                }
                            }
                        }
                    } else {
                        // Pass to thread
                        // It's a text message that has to be handled. If a user has more than one active game
                        // I have to ask him which one
                    }
                } // ignoring other message kinds since they're useless for us
            }  else if let UpdateKind::CallbackQuery(qry) = update.kind {
                use threading::*;
                let data: Vec<String> = qry.data.unwrap().split(":").map(|x| x.to_owned()).collect();
                println!("{:?}", data);
                let command = data[0].as_str();
                match command {
                    "init_game" => {
                        let params = data[1].clone();
                        let game_id = ulid::Ulid::new().to_string();
                        let index = params.parse::<usize>().unwrap();
                        let (sender, reciever) = mpsc::sync_channel(10);
                        let game = Box::leak(playable_games[index].get_new_instance());
                        sender.send(ThreadMessage::AddPlayer(primitives::Player{id: qry.from.id.into(), name: utils::get_user_name(&qry.from.first_name, &qry.from.last_name)})).unwrap();
                        player_games.insert(qry.from.id, game_id.clone());
                        game_channel.insert(game_id.clone(), sender);
                        game_last_played.insert(game_id.clone(), std::time::Instant::now());
                        client.send_message((format!("Per invitare altre persone condividi questo link: https://t.me/giocoacartebot?start={}", game_id), qry.from.id).into());
                        std::thread::spawn(move || {
                            let client = telegram::Telegram::init();
                            let game = game;
                            game.init();
                            let mut game_is_running = true;
                            while game_is_running {
                                let message = reciever.recv().unwrap();
                                let status = match message {
                                    ThreadMessage::AddPlayer(p) => vec![game.add_player(p.clone()).unwrap_or_else(|x| primitives::GameStatus::NotifyUser(p, x.to_owned()))],
                                    ThreadMessage::Start => vec![game.start(), primitives::GameStatus::NotifyRoom(game.get_status())],
                                    ThreadMessage::HandleMove(p, c) => {
                                        let mut tmp = game.handle_move(&p, c);
                                        tmp.push(primitives::GameStatus::NotifyRoom(game.get_status()));
                                        tmp
                                    },
                                    ThreadMessage::Kill => {
                                        break;
                                    },
                                    ThreadMessage::Ping => {vec![]},
                                };
                                for status in &status {
                                    if let primitives::GameStatus::GameEnded = status {
                                        game_is_running = false;
                                    }
                                }
                                for i in utils::compact_messages(status.iter().map(|x| x.dispatch(game)).flatten().collect::<Vec<telegram::Message>>()) {
                                    client.send_message(i);
                                }
                            }
                        });
                    },
                    "start" => {
                        let player_id = qry.from.id;
                        if let Some(game_id) = player_games.get(&player_id) {
                            if let Some(inst) = game_last_played.get_mut(game_id) {
                                *inst = std::time::Instant::now();
                            }
                            let channel = game_channel.get(game_id).unwrap();
                            channel.send(ThreadMessage::Start).expect("Could not start game");
                        } else {
                            client.send_message(("Gioco non trovato", player_id).into());
                        }
                    },
                    "handle_move" => {
                        let card: primitives::Card = bincode::deserialize(&base64::decode(&data[1]).unwrap()).unwrap();
                        let player_id = qry.from.id;
                        if let Some(game_id) = player_games.get(&player_id) {
                            if let Some(inst) = game_last_played.get_mut(game_id) {
                                *inst = std::time::Instant::now();
                            }
                            let channel = game_channel.get(game_id).unwrap();
                            channel.send(ThreadMessage::HandleMove(primitives::Player{id: qry.from.id.into(), name: utils::get_user_name(&qry.from.first_name, &qry.from.last_name)}, card)).expect("Could not handle move");
                        } else {
                            client.send_message(("Gioco non trovato", player_id).into());
                        }
                    }
                    _ => {}
                }
            }
        }
        // Now check if there are games which need to be terminated
        for (game, time) in game_last_played.iter() {
            if time.elapsed().as_secs() > MAX_GAME_DURATION {
                let channel = game_channel.get(game).unwrap();
                channel.send(threading::ThreadMessage::Kill).unwrap_or_default();
            }
        }
        // Cleanup of dead games
        for (game, channel) in game_channel.iter() {
            if channel.send(threading::ThreadMessage::Ping).is_err() {
                // If the game is dead, the channel can't send the message
                cleanup_list.push(game.clone());
            }
        }
        // Deassosciate the game
        player_games = player_games.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.1).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        game_channel = game_channel.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.0).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        game_last_played = game_last_played.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.0).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        cleanup_list.clear();
    }
}
