use super::telegram::{Message, Button};
use std::collections::HashMap;
use cardgames::primitives::*;

pub fn get_user_name(name: &str, surname: &Option<String>) -> String {
    name.to_owned() + if surname.is_some(){" "} else {""} + &(surname.clone()).unwrap_or_else(|| "".to_owned())
}

pub fn compact_messages(list: Vec<Message>) -> Vec<Message> {
    let mut map: HashMap<i64, Vec<Message>> = HashMap::new();
    for message in list {
        if map.get(&message.chat_id).is_none() {
            map.insert(message.chat_id, vec![]);
        }
        let v = map.get_mut(&message.chat_id).unwrap();
        v.push(message.clone());
    }
    map.iter()
        .map(|x| {
            let concatenated_text = x.1.iter()
                .map(|x| &x.text)
                .fold(String::new(), |acc, x| acc + x + "\n");
            let mut keyboards: Vec<Vec<Vec<Button>>> = x.1.iter()
                .map(|x| x.keyboard.clone())
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();
            let keyboard = if keyboards.is_empty() {
                None
            } else {
                let mut tmp_keyboard = vec![];
                for kbd in keyboards.iter_mut() {
                    tmp_keyboard.append(kbd);
                }
                Some(tmp_keyboard)
            };
            Message{chat_id: *(x.0), text: concatenated_text, keyboard}
        })
        .collect()
}

/// This function routes the status to the right players
pub fn dispatch_game_status(status: GameStatus, game: &dyn Game) -> Vec<super::telegram::Message> {
    match status.clone() {
        // Messages for selected players
        // GameStatus::InProgress(p) => vec![(p, self.clone())],
        GameStatus::WaitingForChoice(p, _) => vec![(p, status.clone()).into()],
        GameStatus::WaitingForChoiceCustomMessage(p, _, _) => vec![(p, status.clone()).into()],
        GameStatus::NotifyUser(p, _) => vec![(p, status.clone()).into()],
        GameStatus::WaitingForPlayers(_, _) => {
            // This closure makes sure that only the game initiator
            // gets the button to start the game.
            //use super::telegram::Message;
            let mut res = vec![];
            let mut players = game.get_players();
            players.reverse();
            let player = players.pop().unwrap();
            let text = Message::from((player.clone(), status.clone())).text;
            res.push((player, status.clone()).into());
            res.append(&mut players.iter().map(|x| (x.clone(), GameStatus::NotifyUser(x.clone(), text.clone())).into()).collect());
            res
        }
        // Everything else will sent to everybody in the game
        _ => game.get_players().iter().map(|x| (x.clone(), status.clone()).into()).collect::<Vec<super::telegram::Message>>()
    }
}
