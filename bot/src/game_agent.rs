use std::collections::HashMap;
use super::threading::ThreadMessage;
use cardgames::primitives;
use super::utils;
use super::telegram::{Telegram, Message};
use cardgames::primitives::Game;

pub fn new_agent(game_tg_client: Telegram, game_index: usize, playable_games: &Vec<Box<dyn Game>>, receiver: std::sync::mpsc::Receiver<ThreadMessage>) {
    let game = Box::leak(playable_games[game_index].get_new_instance());
    std::thread::spawn(move || {
        let mut message_list: HashMap<i64, i64> = HashMap::new();
        let client = game_tg_client;
        let game = game;
        game.init();
        let mut game_is_running = true;
        while game_is_running {
            let message = receiver.recv().unwrap();
            let status = match message {
                ThreadMessage::AddPlayer(p) => vec![game.add_player(p.clone()).unwrap_or_else(|x| primitives::GameStatus::NotifyUser(p, x.to_owned()))],
                ThreadMessage::Start => vec![game.start(), primitives::GameStatus::NotifyRoom(game.get_status())],
                ThreadMessage::HandleMove(p, c) => {
                    let mut tmp = game.handle_move(&p, c);
                    tmp.push(primitives::GameStatus::NotifyRoom(game.get_status()));
                    tmp
                },
                ThreadMessage::HandleStringMessage(from, text) => {
                    for message in game.handle_message(text, from).iter()
                        .map(|x| utils::dispatch_game_status(x.clone(), game))
                        .flatten() // Flatten the double Vec
                        .collect::<Vec<Message>>()
                    {
                        client.send_message(message);
                    }
                    vec![]
                },
                ThreadMessage::Kill => {
                    break;
                },
                ThreadMessage::Ping => {vec![]},
                ThreadMessage::AboutToKill => {vec![primitives::GameStatus::NotifyRoom("Questo gioco sarà terminato per inattività a breve!".to_owned())]},
            };
            for status in &status {
                if let primitives::GameStatus::GameEnded = status {
                    game_is_running = false;
                }
            }
            for i in utils::compact_messages(status.iter()
                .map(|x| utils::dispatch_game_status(x.clone(), game)) // find out who's the recipient of each message
                .flatten() // Flatten the double Vec
                .collect::<Vec<Message>>()
            ) {
                match message_list.get_mut(&i.chat_id) {
                    Some(msg_id) => *msg_id = client.edit_message(i, *msg_id),
                    None => {
                        let user_id = i.chat_id;
                        let msg_id = client.send_message(i);
                        message_list.insert(user_id, msg_id);
                    }
                }
            }
        }
    });
}
