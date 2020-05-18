use std::collections::HashMap;
use super::threading::ThreadMessage;
use super::primitives;
use super::utils;
use super::telegram::{Telegram, Message};
use primitives::Game;

pub fn new_agent(game_tg_client: Telegram, game_index: usize, playable_games: &Vec<Box<dyn Game>>, reciever: std::sync::mpsc::Receiver<ThreadMessage>) {
    let game = Box::leak(playable_games[game_index].get_new_instance());
    std::thread::spawn(move || {
        let mut message_list: HashMap<i64, i64> = HashMap::new();
        let client = game_tg_client;
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
                ThreadMessage::AboutToKill => {vec![primitives::GameStatus::NotifyRoom("Questo gioco sarà terminto per inattività a breve!".to_owned())]},
            };
            for status in &status {
                if let primitives::GameStatus::GameEnded = status {
                    game_is_running = false;
                }
            }
            for i in utils::compact_messages(status.iter().map(|x| x.dispatch(game)).flatten().collect::<Vec<Message>>()) {
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