use super::primitives::*;
use rand::seq::SliceRandom;

pub fn get_card_name(card: &Card) -> String {
    let c_type = match card.0 {
        CardType::Jack => "Fante".to_owned(),
        CardType::Queen => "Cavallo".to_owned(),
        CardType::King => "Re".to_owned(),
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