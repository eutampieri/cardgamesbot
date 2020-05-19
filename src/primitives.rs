use std::cmp::{PartialEq, Eq};
use std::hash::Hash;
use serde::{Serialize, Deserialize};

pub enum CardDeckType {
    Briscola,
    Poker,
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CardType {
    Numeric(u8),
    King,
    Queen, // Cavallo in Briscola
    Jack, // Fante in Briscola
}
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum CardSuit {
    Spade,
    Coppe,
    Denari,
    Bastoni,
}
impl From<&CardSuit> for String {
    fn from(s: &CardSuit) -> Self {
        match s {
            CardSuit::Bastoni => "🥢",
            CardSuit::Spade => "🗡 ",
            CardSuit::Coppe => "🏆",
            CardSuit::Denari => "💰"
        }.to_owned()
    }
}

pub type Card = (CardType, CardSuit);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Player {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum GameStatus {
    RoundWon(Player),
    GameEnded,
    InProgress(Player),
    WaitingForPlayers(bool),
    WaitingForChoice(Player, Vec<Card>),
    InvalidMove(&'static str),
    WaitingForChoiceCustomMessage(Player, Vec<Card>, &'static str),
    NotifyUser(Player, String),
    NotifyRoom(String),
    CardPlayed(Player, Card),
}

pub type CardDeck = Vec<Card>;

pub trait Game: Send {
    /// Initialise the game (i.e. prepare the deck and so on)
    fn init(&mut self);
    /// Get the game's name
    fn get_name(&self) -> &str;
    /// Which set does the game use? Briscola or Poker?
    fn get_card_set(&self) -> CardDeckType;
    /// Get the range in which the game can be played
    fn get_num_players(&self) -> std::ops::Range<u8>;
    /// The implementor of the game logic
    fn handle_move(&mut self, by: &Player, card: Card) -> Vec<GameStatus>;
    /// The points associated to each card
    fn get_card_rank(card: &CardType) -> fraction::Fraction where Self: Sized;
    fn get_card_sorting_rank(card: &CardType) -> u8 where Self: Sized;
    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str>;
    fn get_next_player(&self) -> Option<Player>;
    fn start(&mut self) -> GameStatus;
    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::Fraction)>;
    fn get_status(&self) -> String;
    fn get_players(&self) -> Vec<Player>;
    fn get_new_instance(&self) -> Box<dyn Game>;
}

pub type DispatchableStatus = (Player, GameStatus);

impl GameStatus {
    /// This function routes the status to the right players
    pub fn dispatch(&self, game: &dyn Game) -> Vec<super::telegram::Message> {
        match self.clone() {
            // Messages for selected players
            // GameStatus::InProgress(p) => vec![(p, self.clone())],
            GameStatus::WaitingForChoice(p, _) => vec![(p, self.clone()).into()],
            GameStatus::WaitingForChoiceCustomMessage(p, _, _) => vec![(p, self.clone()).into()],
            GameStatus::NotifyUser(p, _) => vec![(p, self.clone()).into()],
            GameStatus::WaitingForPlayers(_) => {
                // This closure makes sure that only the game initiator
                // gets the button to start the game.
                use super::telegram::Message;
                let mut res = vec![];
                let mut players = game.get_players();
                players.reverse();
                let player = players.pop().unwrap();
                let text = Message::from((player.clone(), self.clone())).text;
                res.push((player, self.clone()).into());
                res.append(&mut players.iter().map(|x| (x.clone(), GameStatus::NotifyUser(x.clone(), text.clone())).into()).collect());
                res
            }
            // Everything else will sent to everybody in the game
            _ => game.get_players().iter().map(|x| (x.clone(), self.clone()).into()).collect::<Vec<super::telegram::Message>>()
        }
    }
}
