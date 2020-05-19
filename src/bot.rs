use std::collections::HashMap;
use super::telegram::Telegram;
use super::primitives::Game;
use super::*;
use super::game_agent;

fn add_player_to_game(
    game_id: String,
    client: &Telegram,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    from: telegram_bot_raw::types::chat::User
) {
    client.send_message(("Provo ad aggiungerti alla partita...", from.id).into());
    // Check wether the user is already playing a game
    if player_games.contains_key(&from.id) {
        // The user can't join two games at the same time
        client.send_message(("Non puoi unirti a più partite contemporaneamente", from.id).into());
    } else {
        if let Some(ch) = game_channel.get(&game_id) {
            player_games.insert(from.id, game_id.clone());
            ch.send(threading::ThreadMessage::AddPlayer(primitives::Player{id: from.id.into(), name: utils::get_user_name(&from.first_name, &from.last_name)})).unwrap();
        } else {
            client.send_message(("Gioco non trovato!", from.id).into());
        }
    }
}

fn init_game(
    game_index: usize,
    from: telegram_bot_raw::types::chat::User,
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram
) {
    use threading::ThreadMessage;
    let game_id = ulid::Ulid::new().to_string();
    let (sender, reciever) = mpsc::sync_channel(10);
    sender.send(ThreadMessage::AddPlayer(primitives::Player{id: from.id.into(), name: utils::get_user_name(&from.first_name, &from.last_name)})).unwrap();
    player_games.insert(from.id, game_id.clone());
    game_channel.insert(game_id.clone(), sender);
    game_last_played.insert(game_id.clone(), std::time::Instant::now());
    client.send_message((format!("Per invitare altre persone condividi questo link: https://t.me/giocoacartebot?start={}", game_id), from.id).into());
    let game_tg_client = client.clone();
    game_agent::new_agent(game_tg_client, game_index, playable_games, receiver);
}

fn handle_callback_query(
    qry: telegram_bot_raw::types::callback_query::CallbackQuery,
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram
) {
    use threading::*;
    //let qry_id: String = qry.id.into;
    //client.ack_callback_query(&format!("{}", qry.id));
    let data: Vec<String> = qry.data.unwrap().split(":").map(|x| x.to_owned()).collect();
    let command = data[0].as_str();
    match command {
        "init_game" => {
            let params = data[1].clone();
            let index = params.parse::<usize>().unwrap();
            init_game(index, qry.from, playable_games, player_games, game_channel, game_last_played, client);
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

fn handle_update(
    update: telegram_bot_raw::types::Update,
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram
) {
    use telegram_bot_raw::types::update::UpdateKind;
    use telegram_bot_raw::types::message::MessageKind;
    if let UpdateKind::Message(msg) = update.kind {
        if let MessageKind::Text{data, entities} = msg.kind {
            drop(entities); // Silence the stupid warning and free some RAM
            if data.contains("/start") {
                let pieces: Vec<String> = data.split(" ").map(|x| x.to_owned()).collect();
                if pieces.len() == 1 {
                    client.send_message(("Ciao! A che gioco vuoi giocare?", msg.from.id, playable_games).into());
                } else {
                    let game_id = pieces[1].clone();
                    add_player_to_game(game_id, client, player_games, game_channel, msg.from);
                }
            } else {
                // Pass to thread
                // It's a text message that has to be handled. If a user has more than one active game
                // I have to ask him which one
            }
        } // ignoring other message kinds since they're useless for us
    }  else if let UpdateKind::CallbackQuery(qry) = update.kind {
        handle_callback_query(qry, playable_games, player_games, game_channel, game_last_played, client);
    }
}

pub fn main_bot_logic(
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram
) {
    let mut cleanup_list: Vec<String> = Vec::new();
    loop {
        for update in client.get_updates() {
            handle_update(update, playable_games, player_games, game_channel, game_last_played, client);
        }
        // Now check if there are games which need to be terminated
        for (game, time) in game_last_played.iter() {
            if time.elapsed().as_secs() > MAX_GAME_DURATION {
                let channel = game_channel.get(game).unwrap();
                channel.send(threading::ThreadMessage::Kill).unwrap_or_default();
            } else if time.elapsed().as_secs() > (MAX_GAME_DURATION as f64 * 0.9) as u64 {
                let channel = game_channel.get(game).unwrap();
                channel.send(threading::ThreadMessage::AboutToKill).unwrap_or_default();
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
        *player_games = player_games.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.1).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        *game_channel = game_channel.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.0).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        *game_last_played = game_last_played.iter()
            .filter(|x| cleanup_list.iter().position(|y| y==x.0).is_none())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect();
        cleanup_list.clear();
    }

}
