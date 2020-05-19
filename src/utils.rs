use super::primitives::*;
use rand::seq::SliceRandom;
use super::telegram::{Message, Button};
use std::collections::HashMap;

pub fn get_card_name(card: &Card) -> String {
    let c_type = match card.0 {
        CardType::Jack => "🚶‍♂️".to_owned(),
        CardType::Queen => "🐴".to_owned(),
        CardType::King => "🤴".to_owned(),
        CardType::Numeric(x) => match x {
            1 => "Asso".to_owned(),
            _ => format!("{}", x)
        }
    };
    format!("{} di {}", c_type, String::from(&card.1))
}

pub fn random_deck(of_type: CardDeckType) -> Vec<Card> {
    match of_type {
        CardDeckType::Briscola => {
            let mut rng = rand::thread_rng();
            let mut deck_raw: Vec<u8> = (0..40).collect();
            deck_raw.shuffle(&mut rng);
            let deck: Vec<Card> = deck_raw.iter().map(|x| {
                let value: u8 = x % 10;
                // Ricordarsi di aggiungere 1
                let suit = match x - value {
                    0 => CardSuit::Bastoni,
                    10 => CardSuit::Coppe,
                    20 => CardSuit::Denari,
                    _ => CardSuit::Spade
                };
                let c_type = match value {
                    9 => CardType::King,
                    8 => CardType::Queen,
                    7 => CardType::Jack,
                    _ => CardType::Numeric(value + 1)
                };
                (c_type, suit)
            }).collect();
            deck
        },
        CardDeckType::Poker => unimplemented!()
    }
}

pub fn zero() -> fraction::Fraction {
    fraction::Fraction::new(0u8, 1u8)
}
pub fn one() -> fraction::Fraction {
    fraction::Fraction::new(1u8, 1u8)
}

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
