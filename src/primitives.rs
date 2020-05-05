use std::cmp::{PartialEq, Eq};
use std::hash::Hash;

pub enum CardDeckType {
    Briscola,
    Poker,
}
#[derive(Clone, PartialEq, Debug)]
pub enum CardType {
    Numeric(u8),
    King,
    Queen, // Cavallo in Briscola
    Jack, // Fante in Briscola
}
#[derive(Clone, PartialEq, Debug)]
pub enum CardSuit {
    Spade,
    Coppe,
    Denari,
    Bastoni,
}
impl From<&CardSuit> for String {
    fn from(s: &CardSuit) -> Self {
        match s {
            CardSuit::Bastoni => "Bastoni",
            CardSuit::Spade => "Spade",
            CardSuit::Coppe => "Coppe",
            CardSuit::Denari => "Denari"
        }.to_owned()
    }
}

pub type Card = (CardType, CardSuit);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Player {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub enum GameStatus {
    RoundWon(Player, Player),
    GameEnded,
    InProgress(Player),
    WaitingForPlayers(bool),
    WaitingForChoice(Player, Vec<Card>),
    InvalidMove(&'static str),
    WaitingForChoiceCustomMessage(Player, Vec<Card>, &'static str),
    GameReady,
    NotifyUser(Player, String),
}

pub type CardDeck = Vec<Card>;

pub trait Game {
    /// Initialise the game (i.e. prepare the deck and so on)
    fn init() -> Self;
    /// Get the game's name
    fn get_name(&self) -> &str;
    /// Which set does the game use? Briscola or Poker?
    fn get_card_set(&self) -> CardDeckType;
    /// Get the range in which the game can be played
    fn get_num_players(&self) -> std::ops::Range<u8>;
    /// The implementor of the game logic
    fn handle_move(&mut self, by: &Player, card: Card) -> Vec<GameStatus>;
    /// The points associated to each card
    fn get_card_rank(card: &CardType) -> fraction::Fraction;
    fn get_card_sorting_rank(card: &CardType) -> u8;
    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str>;
    fn get_next_player(&self) -> Option<Player>;
    fn start(&mut self) -> GameStatus;
    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::Fraction)>;
    fn get_status(&self) -> String;
}
