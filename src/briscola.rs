use super::primitives::*;
use std::collections::hash_map::*;
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct Briscola {
    /// The cards on the table, associated with the player
    table: Vec<(Player, Card)>,
    /// List of players
    players: Vec<Player>,
    /// Card in hand for each player
    in_hand: HashMap<Player, Vec<Card>>,
    /// Teams
    teams: Vec<Vec<Player>>,
    player_team: HashMap<Player, usize>,
    won_cards: Vec<Vec<Card>>,
    deck: CardDeck,
    briscola: CardSuit,
    next_player: Option<Player>,
}

impl Game for Briscola {
    fn get_card_rank(card: &CardType) -> fraction::Fraction {
        fraction::Fraction::new(match card {
            CardType::Jack => 2,
            CardType::Queen => 3,
            CardType::King => 4,
            CardType::Numeric(x) => {
                match x {
                    1 => 11,
                    3 => 10,
                    _ => 0
                }
            }
        } as u8, 1u8)
    }
    fn get_card_sorting_rank(card: &CardType) -> u8 {
        match card {
            CardType::Jack => 6,
            CardType::Queen => 7,
            CardType::King => 8,
            CardType::Numeric(x) => {
                match x {
                    1 => 10,
                    3 => 9,
                    7 => 5,
                    6 => 4,
                    5 => 3,
                    4 => 2,
                    2 => 1,
                    _ => 0
                }
            }
        }
    }
    fn init() -> Self {
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
        let mut teams = Vec::new();
        teams.push(Vec::new());
        teams.push(Vec::new());
        let mut wc = Vec::new();
        wc.push(Vec::new());
        wc.push(Vec::new());
        Briscola{
            table: Vec::new(),
            players: Vec::new(),
            in_hand: HashMap::new(),
            teams: teams,
            player_team: HashMap::new(),
            won_cards: wc,
            deck: deck.clone(),
            briscola: deck.first().unwrap().1.clone(),
            next_player: None,
        }
    }
    fn get_name(&self) -> &str {
        "Briscola"
    }
    fn get_card_set(&self) -> CardDeckType {
        CardDeckType::Briscola
    }
    fn get_num_players(&self) -> std::ops::Range<u8> {
        2..4
    }
    fn handle_move(&mut self, by: &Player, card: Card) -> Vec<GameStatus> {
        // FIXME bloccare mossa se non è il tuo turno
        let player_index = self.players.iter().position(|x| x == by).unwrap();
        let next_player = self.players.clone()[(player_index + 1) % self.players.len()].clone();
        self.next_player = Some(next_player.clone());
        // Tolgo la carta dalle carte in mano
        let card_index = self.in_hand.get(by).unwrap().iter().position(|x| x.clone() == card).expect("Non trovo la carta");
        self.in_hand.get_mut(by).unwrap().remove(card_index);
        // E la metto sul tavolo
        self.table.push((by.clone(), card));
        if self.table.len() == self.players.len() { // Se tutti hanno messo una carta
            // Determinare la carta vincente
            let mut winner = (self.table[0]).clone().0;
            let mut winning_suit = &(&(self.table[0]).1).clone().1;
            let mut max: i32 = -1;
            for (player, card) in &self.table {
                if card.1 == self.briscola && winning_suit != &self.briscola {
                    winning_suit = &self.briscola;
                    max = Self::get_card_sorting_rank(&card.0) as i32;
                    winner = player.clone();
                }
                if &card.1 == winning_suit && Self::get_card_sorting_rank(&card.0) as i32 > max {
                    max = Self::get_card_sorting_rank(&card.0) as i32;
                    winner = player.clone();
                }
            }
            // Abbiamo determinato chi ha vinto la mano, assegnamogliela
            self.won_cards[*self.player_team.get(&winner).unwrap()].append(&mut self.table.iter().map(|x| x.1.clone()).collect());
            self.table.clear(); // Just in case...
            if self.deck.len() == 0 {
                vec![GameStatus::GameEnded]
            } else {
                // Do le carte
                for player in &self.players {
                    // FIXME dare le carte in ordine giusto in base alla fittoria
                    self.in_hand.get_mut(player).unwrap().push(self.deck.pop().unwrap());
                }
                self.next_player = Some(winner.clone());
                vec![GameStatus::WaitingForChoice(winner.clone(), self.in_hand.get(&next_player).unwrap().clone()), GameStatus::RoundWon(winner, next_player)]
            }
        } else {
            self.next_player = Some(next_player.clone());
            vec![GameStatus::WaitingForChoice(next_player.clone(), self.in_hand.get(&next_player).unwrap().clone()), GameStatus::InProgress(next_player)]
        }
    }
    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str> {
        // TODO ritornare un errore se la partita è già iniziata
        if self.players.len() <= self.get_num_players().end as usize {
            if self.players.len() == 0 {
                self.next_player = Some(player.clone());
            }
            let is_ready = self.players.len() <= self.get_num_players().end as usize && self.get_num_players().start as usize <= self.players.len();
            // Aggiungo il giocatore
            self.players.push(player.clone());
            // Do le carte
            let mut hand = Vec::new();
            for _ in 0..3 {
                hand.push(self.deck.pop().unwrap());
            }
            self.in_hand.insert(player.clone(), hand);
            // Lo assegno ad un team
            self.teams[self.players.len() % 2].push(player.clone());
            self.player_team.insert(player, self.players.len() % 2);
            Ok(GameStatus::WaitingForPlayers(is_ready))
        } else {
            Err("Il gioco è pieno")
        }
    }
    fn get_next_player(&self) -> Option<Player> {
        self.next_player.clone()
    }
    fn start(&mut self) -> GameStatus {
        // FIXME Dare le carte qui e non quando si aggiungono i giocatori
        // FIXME decidere la briscola dopo aver rimosso la carta
        if self.players.len() == 3 {
            // FIXME controllare che non sia di briscola
            // Spostare il terzo giocatore in un team a se stante e togliere una carta
            if let Some(i) = self.deck.iter().position(|x| x.0 == CardType::Numeric(2)) {
                self.deck.remove(i);
            } else if let Some(i) = self.deck.iter().position(|x| x.0 == CardType::Numeric(4)) {
                self.deck.remove(i);
            } else if let Some(i) = self.deck.iter().position(|x| x.0 == CardType::Numeric(5)) {
                self.deck.remove(i);
            } else {
                // Terminiamo in anticipo il gioco, non bellissimo
                // Non si può mica togliere un sei eh, o un sette, altrimenti come si fa a giocare?
                // Ma piutòst che gnit, l'è mej piutòst
                return GameStatus::GameEnded;
            }
            self.teams.push(Vec::new());
            self.won_cards.push(Vec::new());
            let moved_player = self.teams[0].pop().unwrap();
            self.teams[2].push(moved_player);
            self.player_team.insert(self.teams[2][0].clone(), self.players.len() % 2);
        }
        let player = self.players[0].clone();
        GameStatus::WaitingForChoice(player.clone(), self.in_hand.get(&player).unwrap().clone())
    }
    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::Fraction)> {
        self.teams.iter()
            .zip(self.won_cards.iter())
            .map(|x| {
                let player_lst = x.0.clone();
                let score = x.1.iter()
                    .map(|x| Self::get_card_rank(&x.0))
                    .sum();
                (player_lst, score)
            })
            .collect()
    }
}
