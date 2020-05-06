use std::env;
use itertools::Itertools;
use super::primitives;

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

impl Message for primitives::DispatchableStatus {
    fn get_chat_id(&self) -> i64 {
        self.0.id
    }
    fn get_text(&self) -> String {
        use primitives::GameStatus;
        match self.1 {
            GameStatus::GameEnded => 
        }
    }
}
