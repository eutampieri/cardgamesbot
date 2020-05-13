mod primitives;
mod briscola;
mod beccaccino;
mod utils;
mod telegram;

use primitives::Game;
use std::sync::mpsc;

fn main() {
    let mut playable_games: Vec<Box<dyn Game>> = Vec::new();
    // List of playable games
    playable_games.push(Box::from(briscola::Briscola::init()));
    playable_games.push(Box::from(beccaccino::Beccaccino::init()));

    println!("Starting CardGamesBot...");
    let mut client = telegram::Telegram::init();
    loop {
        use telegram_bot_raw::types::update::UpdateKind;
        use telegram_bot_raw::types::message::MessageKind;
        for update in client.get_updates() {
            if let UpdateKind::Message(msg) = update.kind {
                if let MessageKind::Text{data, entities} = msg.kind {
                    drop(entities); // Silece the stupid warning and free some RAM
                    if data.contains("/start") {
                        let pieces: Vec<String> = data.split(" ").map(|x| x.to_owned()).collect();
                        if pieces.len() == 1 {
                            client.send_message(("Ciao! A che gioco vuoi giocare?", msg.from.id, &playable_games));
                        } else {
                            let game_id = pieces[1].clone();
                            client.send_message((format!("Ti sto per aggiungere al gioco {}", game_id), msg.from.id));
                        }
                    } else {
                        // Pass to thread
                        // It's a text message that has to be handled. If a user has more than one active game
                        // I have to ask him which one
                    }
                } // ignoring other message kinds since they're useless for us
            }  else if let UpdateKind::CallbackQuery(qry) = update.kind {
                let data: Vec<String> = qry.data.unwrap().split(":").map(|x| x.to_owned()).collect();
                let command = data[0].as_str();
                let params = data[1].clone();
                match command {
                    "init_game" => {
                        let index = params.parse::<usize>().unwrap();
                        let (sender, reciever) = mpsc::sync_channel(10);
                        //let game = playable_games[index].clone().get_new_instance();
                        //assert_ne!(game, playable_games[index]);
                        sender.send(format!("{}", 1)).unwrap();
                        /*std::thread::spawn(move |game, reciever| {

                        });*/
                    },
                    _ => {}
                }
            }
        }
    }
}
