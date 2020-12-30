use super::game_agent;
use super::telegram::Telegram;
use super::*;
use cardgames::primitives::Game;
use std::collections::HashMap;

fn add_player_to_game(
    game_id: String,
    client: &Telegram,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    from: telegram_bot_raw::types::chat::User,
) {
    client.send_message(("Provo ad aggiungerti alla partita...", from.id).into());
    // Check wether the user is already playing a game
    if player_games.contains_key(&from.id) {
        // The user can't join two games at the same time
        client.send_message(("Non puoi unirti a più partite contemporaneamente", from.id).into());
    } else if let Some(ch) = game_channel.get(&game_id) {
        player_games.insert(from.id, game_id.clone());
        ch.send(threading::ThreadMessage::AddPlayer(
            cardgames::primitives::Player {
                id: from.id.into(),
                name: utils::get_user_name(&from.first_name, &from.last_name),
            },
        ))
        .unwrap();
    } else {
        client.send_message(("Gioco non trovato!", from.id).into());
    }
}

fn handle_string_message(
    game_id: &String,
    client: &Telegram,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    from: telegram_bot_raw::types::chat::User,
    text: String,
) {
    // Check wether the user is already playing a game
    if let Some(ch) = game_channel.get(game_id) {
        ch.send(threading::ThreadMessage::HandleStringMessage(
            cardgames::primitives::Player {
                id: from.id.into(),
                name: utils::get_user_name(&from.first_name, &from.last_name),
            },
            text,
        ))
        .unwrap();
    } else {
        client.send_message(("Gioco non trovato!", from.id).into());
    }
}

fn init_game(
    game_index: usize,
    from: telegram_bot_raw::types::chat::User,
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram,
) {
    use threading::ThreadMessage;
    let game_id = ulid::Ulid::new().to_string();
    let (sender, receiver) = mpsc::sync_channel(10);
    sender
        .send(ThreadMessage::AddPlayer(cardgames::primitives::Player {
            id: from.id.into(),
            name: utils::get_user_name(&from.first_name, &from.last_name),
        }))
        .unwrap();
    player_games.insert(from.id, game_id.clone());
    game_channel.insert(game_id.clone(), sender);
    game_last_played.insert(game_id.clone(), std::time::Instant::now());
    client.send_message(
        (
            format!(
                "Per invitare altre persone condividi questo link: https://t.me/{}?start={}",
                &client.username, game_id
            ),
            from.id,
        )
            .into(),
    );
    let game_tg_client = client.clone();
    game_agent::new_agent(game_tg_client, game_index, playable_games, receiver);
}

fn try_start_game(
    player_id: telegram_bot_raw::types::refs::UserId,
    player_games: &HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    game_channel: &HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    client: &Telegram,
) {
    use threading::ThreadMessage;
    if let Some(game_id) = player_games.get(&player_id) {
        if let Some(inst) = game_last_played.get_mut(game_id) {
            *inst = std::time::Instant::now();
        }
        let channel = game_channel.get(game_id).unwrap();
        channel
            .send(ThreadMessage::Start)
            .expect("Could not start game");
    } else {
        client.send_message(("Gioco non trovato", player_id).into());
    }
}
fn try_handle_move(
    card: cardgames::primitives::Card,
    from: telegram_bot_raw::types::chat::User,
    player_games: &HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    game_channel: &HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    client: &Telegram,
) {
    if let Some(game_id) = player_games.get(&from.id) {
        if let Some(inst) = game_last_played.get_mut(game_id) {
            *inst = std::time::Instant::now();
        }
        let channel = game_channel.get(game_id).unwrap();
        channel
            .send(threading::ThreadMessage::HandleMove(
                cardgames::primitives::Player {
                    id: from.id.into(),
                    name: utils::get_user_name(&from.first_name, &from.last_name),
                },
                card,
            ))
            .expect("Could not handle move");
    } else {
        client.send_message(("Gioco non trovato", from.id).into());
    }
}

fn handle_callback_query(
    qry: telegram_bot_raw::types::callback_query::CallbackQuery,
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram,
) {
    //let qry_id: String = qry.id.into;
    //client.ack_callback_query(&format!("{}", qry.id));
    let data: Vec<String> = qry.data.unwrap().split(":").map(|x| x.to_owned()).collect();
    let command = data[0].as_str();
    match command {
        "init_game" => {
            let params = data[1].clone();
            let index = params.parse::<usize>().unwrap();
            init_game(
                index,
                qry.from,
                playable_games,
                player_games,
                game_channel,
                game_last_played,
                client,
            );
        }
        "start" => {
            let player_id = qry.from.id;
            try_start_game(
                player_id,
                player_games,
                game_last_played,
                game_channel,
                client,
            );
        }
        "handle_move" => {
            let card: cardgames::primitives::Card =
                bincode::deserialize(&base64::decode(&data[1]).unwrap()).unwrap();
            try_handle_move(
                card,
                qry.from,
                player_games,
                game_last_played,
                game_channel,
                client,
            );
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
    client: &mut Telegram,
) {
    use telegram_bot_raw::types::message::MessageKind;
    use telegram_bot_raw::types::update::UpdateKind;
    if let UpdateKind::Message(msg) = update.kind {
        if let MessageKind::Text { data, entities } = msg.kind {
            drop(entities); // Silence the stupid warning and free some RAM
            if data.contains("/start") {
                let pieces: Vec<String> = data.split(" ").map(|x| x.to_owned()).collect();
                if pieces.len() == 1 {
                    client.send_message(
                        (
                            "Ciao! A che gioco vuoi giocare?",
                            msg.from.id,
                            playable_games,
                        )
                            .into(),
                    );
                } else {
                    let game_id = pieces[1].clone();
                    add_player_to_game(game_id, client, player_games, game_channel, msg.from);
                }
            } else if data == "/commit" {
                use git_version::git_version;
                client.send_message(
                    (
                        format!("This instance is running on {}", git_version!()),
                        msg.from.id,
                    )
                        .into(),
                );
            } else {
                // Pass to thread
                // It's a text message that has to be handled. If a user has more than one active game
                // I have to ask him which one
                if let Some(game_id) = player_games.get(&msg.from.id) {
                    handle_string_message(game_id, client, game_channel, msg.from, data);
                }
            }
        } // ignoring other message kinds since they're useless for us
    } else if let UpdateKind::CallbackQuery(qry) = update.kind {
        handle_callback_query(
            qry,
            playable_games,
            player_games,
            game_channel,
            game_last_played,
            client,
        );
    }
}

fn handle_game_termination(
    game_last_played: &HashMap<String, std::time::Instant>,
    game_channel: &HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
) {
    for (game, time) in game_last_played.iter() {
        if time.elapsed().as_secs() > MAX_GAME_DURATION {
            let channel = game_channel.get(game).unwrap();
            channel
                .send(threading::ThreadMessage::Kill)
                .unwrap_or_default();
        } else if time.elapsed().as_secs() > (MAX_GAME_DURATION as f64 * 0.9) as u64 {
            let channel = game_channel.get(game).unwrap();
            channel
                .send(threading::ThreadMessage::AboutToKill)
                .unwrap_or_default();
        }
    }
}
fn get_dead_games(
    game_channel: &HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
) -> Vec<String> {
    let mut cleanup_list: Vec<String> = Vec::new();
    for (game, channel) in game_channel.iter() {
        if channel.send(threading::ThreadMessage::Ping).is_err() {
            // If the game is dead, the channel can't send the message
            cleanup_list.push(game.clone());
        }
    }
    cleanup_list
}
fn purge_dead_games(
    cleanup_list: Vec<String>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
) {
    *player_games = player_games
        .iter()
        .filter(|x| cleanup_list.iter().position(|y| y == x.1).is_none())
        .map(|x| (x.0.clone(), x.1.clone()))
        .collect();
    *game_channel = game_channel
        .iter()
        .filter(|x| cleanup_list.iter().position(|y| y == x.0).is_none())
        .map(|x| (x.0.clone(), x.1.clone()))
        .collect();
    *game_last_played = game_last_played
        .iter()
        .filter(|x| cleanup_list.iter().position(|y| y == x.0).is_none())
        .map(|x| (x.0.clone(), x.1.clone()))
        .collect();
}

pub fn main_bot_logic(
    playable_games: &Vec<Box<dyn Game>>,
    player_games: &mut HashMap<telegram_bot_raw::types::refs::UserId, String>,
    game_channel: &mut HashMap<String, std::sync::mpsc::SyncSender<threading::ThreadMessage>>,
    game_last_played: &mut HashMap<String, std::time::Instant>,
    client: &mut Telegram,
) {
    loop {
        for update in client.get_updates() {
            handle_update(
                update,
                playable_games,
                player_games,
                game_channel,
                game_last_played,
                client,
            );
        }
        handle_game_termination(game_last_played, game_channel);
        purge_dead_games(
            get_dead_games(game_channel),
            player_games,
            game_channel,
            game_last_played,
        );
    }
}
