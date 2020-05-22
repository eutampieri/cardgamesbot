use std::env;
use itertools::Itertools;
use super::primitives;
use super::utils;
use serde::Deserialize;
use primitives::Game;

#[derive(Debug, Clone)]
pub struct Button {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub chat_id: i64,
    pub text: String,
    pub keyboard: Option<Vec<Vec<Button>>>,
}

impl Message {
    fn get_raw(&self) -> String {
        let mut res = String::new();
        res.push_str("chat_id=");
        res.push_str(pct_str::PctString::encode(format!("{}", self.chat_id).chars(), pct_str::URIReserved).as_str());
        res.push_str("&text=");
        res.push_str(pct_str::PctString::encode(self.text.chars(), pct_str::URIReserved).as_str());
        if let Some(kbd) = &self.keyboard {
            res.push_str("&reply_markup=");
            let json = format!("{{\"inline_keyboard\":[{}]}}",
                kbd.iter()
                    .map(|x| format!("[{}]", x.iter()
                        .map(|y| format!("{{\"text\": \"{}\", \"callback_data\": \"{}\"}}", y.text, y.id))
                        .join(", ")
                    )
                )
                .join(", "));
            res.push_str(pct_str::PctString::encode(json.chars(), pct_str::URIReserved).as_str());
        }
        res
    }
}

#[derive(Clone)]
pub struct Telegram {
    token: String,
    last_id: Option<u64>,
}

impl Telegram {
    pub fn init() -> Self {
        Self{
            token: env::var("TG_BOT_TOKEN").expect("Run specifying the env var TG_BOT_TOKEN"),
            last_id: None
        }
    }

    pub fn send_message(&self, message: Message) -> i64{
        #[derive(Deserialize, Debug)]
        struct Response {
            ok: bool,
            result: telegram_bot_raw::types::message::RawMessage,
        }
        let res = ureq::post(&format!("https://api.telegram.org/bot{}/sendMessage", self.token))
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&message.get_raw()).into_string().unwrap();
        let parsed: Response = serde_json::from_str(&res).unwrap();
        parsed.result.message_id
    }

    pub fn edit_message(&self, message: Message, id: i64) -> i64 {
        ureq::post(&format!("https://api.telegram.org/bot{}/deleteMessage", self.token))
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&format!("chat_id={}&message_id={}", message.chat_id, id));
        self.send_message(message)
    }
    pub fn get_updates(&mut self) -> Vec<telegram_bot_raw::types::update::Update> {
        #[derive(Deserialize, Clone)]
        struct Result {
            result: Vec<telegram_bot_raw::types::update::Update>
        }
        let res = ureq::get(&format!(
                "https://api.telegram.org/bot{}/getUpdates?timeout=60{}",
                self.token,
                if let Some(x) = self.last_id {
                    format!("&offset={}", x+1)
                } else { "".to_owned() }
            ))
            .timeout_read(70_000)
            .call()
            .into_string().unwrap();
        let parsed: Result = serde_json::from_str(&res).unwrap();
        if parsed.result.len() > 0 {
            self.last_id = Some(parsed.clone().result.last().unwrap().id as u64);
        }
        parsed.result
    }
    pub fn ack_callback_query(&self, qry_id: &str) {
        ureq::post(&format!("https://api.telegram.org/bot{}/editMessageText", self.token))
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&format!("callback_query_id={}&show_alert=true", qry_id));
    }
}

/// This function converts a list of cards into an array of buttons
fn deck_of_buttons(cards: Vec<super::primitives::Card>) -> Vec<Vec<Button>> {
    let mut res = vec![];
    // Now add a row every 3 cards
    for _ in (0..cards.len()).skip(3) {
        res.push(vec![]);
    }
    res.push(vec![]);
    // Let's add the cards
    for (i, card) in cards.iter().enumerate() {
        let row_number = i / 3;
        res[row_number].push(Button{
            text: utils::get_card_name(card),
            id: format!("handle_move:{}", base64::encode(bincode::serialize(card).unwrap()))
        //                                                                  ^
        //I'm serializing cards to deserialize later -----------------------|
        });
    }
    res
}

impl From<primitives::DispatchableStatus> for Message {
    fn from(status: primitives::DispatchableStatus) -> Self {
        Self{
            chat_id: status.0.id,
            text: {
                use primitives::GameStatus::*;
                match status.1.clone() {
                    GameEnded => "La partita è finita!".to_owned(),
                    RoundWon(p) => format!("{} ha vinto questo round", p.name),
                    InProgress(p) => format!("Tocca a {}", p.name),
                    WaitingForPlayers(_, p) => format!("{} si è unito alla partita", p.name),
                    WaitingForChoice(_, _) => "Scegli una carta:".to_owned(),
                    InvalidMove(msg) => format!("Questa mossa non è valida! {}", msg),
                    WaitingForChoiceCustomMessage(_, _, msg) => msg.to_string(),
                    NotifyUser(_, msg) => msg,
                    NotifyRoom(msg) => msg,
                    CardPlayed(p, c) => format!("{} ha giocato {}", p.name, utils::get_card_name(&c)),
                }
            },
            keyboard: {
                use primitives::GameStatus::*;
                match status.1.clone() {
                    WaitingForPlayers(ready, _) => {
                        if ready {
                            Some(vec![vec![Button{id: "start".to_owned(), text: "Avvia partita".to_owned()}]])
                        } else {
                            None
                        }
                    },
                    WaitingForChoice(_, cards) => Some(deck_of_buttons(cards)),
                    WaitingForChoiceCustomMessage(_, cards, _) => Some(deck_of_buttons(cards)),
                    _ => None
                }
            },
        }
    }
}

impl From<(&str, telegram_bot_raw::types::refs::UserId)> for Message {
    fn from(tuple: (&str, telegram_bot_raw::types::refs::UserId)) -> Self {
        Self {
            chat_id: tuple.1.into(),
            text: tuple.0.to_owned(),
            keyboard: None,
        }
    }
}
impl From<(String, telegram_bot_raw::types::refs::UserId)> for Message {
    fn from(tuple: (String, telegram_bot_raw::types::refs::UserId)) -> Self {
        Self{
            chat_id: tuple.1.into(),
            text: tuple.0.clone(),
            keyboard: None,
        }
    }
}
impl From<(&str, telegram_bot_raw::types::refs::UserId, &Vec<Box<dyn Game>>)> for Message {
    fn from(tuple: (&str, telegram_bot_raw::types::refs::UserId, &Vec<Box<dyn Game>>)) -> Self {
        Self{
            chat_id: tuple.1.into(),
            text: tuple.0.to_owned(),
            keyboard: {
                Some(tuple.2.iter().enumerate().map(|x| {
                    let range = x.1.get_num_players();
                        vec![Button {
                                id: format!("init_game:{}", x.0),
                                text: format!("{} ({} giocatori)", x.1.get_name(), if range.start == range.end {
                                    format!("{}", range.start)
                                } else {format!("{} - {}", range.start, range.end)})
                            }
                            ]
                        }
                    ).collect())
            }
        }
    }
}
