use std::env;
use itertools::Itertools;
use super::primitives;
use super::utils;

pub struct Button {
    pub id: String,
    pub text: String,
}

pub trait Message {
    fn get_chat_id(&self) -> i64;
    fn get_text(&self) -> String;
    fn get_keyboard(&self) -> Option<Vec<Vec<Button>>>;
    fn get_raw(&self) -> String {
        let mut res = String::new();
        res.push_str("chat_id=");
        res.push_str(pct_str::PctString::encode(format!("{}", self.get_chat_id()).chars(), pct_str::URIReserved).as_str());
        res.push_str("&text=");
        res.push_str(pct_str::PctString::encode(self.get_text().chars(), pct_str::URIReserved).as_str());
        if let Some(kbd) = self.get_keyboard() {
            res.push_str("&reply_markup=");
            let json = format!("\"inline_keyboard\"={{[{}]}}",
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
    fn get_raw_for_edit(&self, id: String) -> String {
        format!("message_id={}&{}",
            pct_str::PctString::encode(id.chars(), pct_str::URIReserved).as_str(),
            self.get_raw()
        )
    }
}

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

    pub fn send_message(&self, message: impl Message) {
        ureq::post(&format!("https://api.telegram.org/bot{}/sendMessage", self.token))
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&message.get_raw());
    }

    pub fn edit_message(&self, message: impl Message, id: String) {
        ureq::post(&format!("https://api.telegram.org/bot{}/editMessageText", self.token))
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&message.get_raw_for_edit(id));
    }

}

/// This function converts a list of cards into an array of buttons
fn deck_of_buttons(cards: Vec<super::primitives::Card>) -> Vec<Vec<Button>> {
    let mut res = vec![];
    // Now add a row every 3 cards
    for _ in (0..cards.len()).skip(3) {
        res.push(vec![]);
    }
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

impl Message for primitives::DispatchableStatus {
    fn get_chat_id(&self) -> i64 {
        self.0.id
    }
    fn get_text(&self) -> String {
        use primitives::GameStatus::*;
        match self.1.clone() {
            GameEnded => "La partita è finita!".to_owned(),
            RoundWon(p) => format!("{} ha vinto questo round", p.name),
            InProgress(p) => format!("Tocca a {}", p.name),
            WaitingForPlayers(_) => format!("In attesa di giocatori..."),
            WaitingForChoice(_, _) => "Scegli una carta:".to_owned(),
            InvalidMove(msg) => format!("Questa mossa non è valida! {}", msg),
            WaitingForChoiceCustomMessage(_, _, msg) => msg.to_string(),
            GameReady => "La partita è pronta!".to_owned(),
            NotifyUser(_, msg) => msg,
            NotifyRoom(msg) => msg,
        }
    }
    fn get_keyboard(&self) -> Option<Vec<Vec<Button>>> {
        use primitives::GameStatus::*;
        match self.1.clone() {
            WaitingForPlayers(ready) => {
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
    }
}

//impl Message for Vec<primitives::DispatchableStatus> {}
