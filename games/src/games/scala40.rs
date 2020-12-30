use crate::primitives::*;
#[derive(Default)]
pub struct Scala40 {
    discarded: Vec<Card>,
    scale: std::collections::HashMap<Player, Vec<Vec<Card>>>,
    /// List of players
    players: Vec<Player>,
    /// Card in hand for each player
    in_hand: std::collections::HashMap<Player, Vec<Card>>,
    /// Teams
    teams: Vec<Vec<Player>>,
    player_team: std::collections::HashMap<Player, usize>,
}

impl Scala40 {
    fn game_has_been_won(&self) -> bool {
        false
    }
}

impl Game for Scala40 {
    fn init(&mut self) {}

    fn get_name(&self) -> &str {
        "Scala 40"
    }

    fn get_card_set(&self) -> CardDeckType {
        CardDeckType::Poker
    }

    fn get_num_players(&self) -> std::ops::Range<u8> {
        2..4
    }

    fn handle_move(&mut self, by: &Player, card: Card) -> Vec<GameStatus> {
        todo!()
    }

    fn get_card_rank(card: &CardType) -> fraction::Fraction
    where
        Self: Sized,
    {
        fraction::Fraction::new(Self::get_card_sorting_rank(card), 1u8)
    }

    fn get_card_sorting_rank(card: &CardType) -> u8
    where
        Self: Sized,
    {
        match card {
            CardType::Numeric(x) => *x,
            CardType::King => 10,
            CardType::Queen => 10,
            CardType::Jack => 10,
            CardType::Jolly => 0,
        }
    }

    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str> {
        todo!()
    }

    fn get_next_player(&self) -> Option<Player> {
        todo!()
    }

    fn start(&mut self) -> GameStatus {
        todo!()
    }

    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::Fraction)> {
        todo!()
    }

    fn get_status(&self) -> String {
        todo!()
    }

    fn get_players(&self) -> Vec<Player> {
        todo!()
    }

    fn get_new_instance(&self) -> Box<dyn Game> {
        todo!()
    }

    fn handle_message(&self, message: String, from: Player) -> Vec<GameStatus> {
        vec![GameStatus::NotifyRoom(format!(
            "{} ha detto: {}",
            from.name, message
        ))]
    }
}
