use std::cmp::{PartialEq, Eq};
use std::hash::Hash;
use serde::{Serialize, Deserialize};

pub type Card = (CardType, CardSuit);

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
    WaitingForPlayers(bool, Player),
    WaitingForChoice(Player, Vec<Card>),
    InvalidMove(&'static str),
    WaitingForChoiceCustomMessage(Player, Vec<Card>, &'static str),
    NotifyUser(Player, String),
    NotifyRoom(String),
    CardPlayed(Player, Card),
}

pub type CardDeck = Vec<Card>;

pub trait Game: Send {
    /// Reinitialise the game (i.e. prepare the deck and so on) after a default instance has been cloned
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
    /// This function gets called when a user sends a text message to the bot.
    /// It should handle the message and pass it to the right users.
    fn handle_message(&self, message: String, from: Player) -> Vec<GameStatus>;
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
