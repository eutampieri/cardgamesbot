use crate::primitives::*;
use crate::utils;
use itertools::Itertools;
use std::collections::hash_map::*;

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
    started: bool,
}

impl Game for Briscola {
    fn get_card_rank(card: &CardType) -> fraction::GenericFraction<u8> {
        fraction::GenericFraction::new(
            match card {
                CardType::Jack => 2,
                CardType::Queen => 3,
                CardType::King => 4,
                CardType::Numeric(x) => match x {
                    1 => 11,
                    3 => 10,
                    _ => 0,
                },
                CardType::Jolly => 0,
            } as u8,
            1u8,
        )
    }
    fn get_card_sorting_rank(card: &CardType) -> u8 {
        match card {
            CardType::Jack => 6,
            CardType::Queen => 7,
            CardType::King => 8,
            CardType::Numeric(x) => match x {
                1 => 10,
                3 => 9,
                7 => 5,
                6 => 4,
                5 => 3,
                4 => 2,
                2 => 1,
                _ => 0,
            },
            CardType::Jolly => 0,
        }
    }
    fn init(&mut self) {
        let deck = utils::random_deck(CardDeckType::Briscola);
        self.deck = deck.clone();
        self.briscola = deck.first().unwrap().1.clone();
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
        // bloccare mossa se non è il tuo turno
        if self.get_next_player().is_none() {
            return vec![GameStatus::InvalidMove("La partita non è ancora iniziata")];
        } else if &self.get_next_player().unwrap() != by {
            return vec![GameStatus::InvalidMove("Non è ancora il tuo turno!")];
        }
        let player_index = self.players.iter().position(|x| x == by).unwrap();
        let next_player = self.players.clone()[(player_index + 1) % self.players.len()].clone();
        self.next_player = Some(next_player.clone());
        // Tolgo la carta dalle carte in mano
        let card_index = self
            .in_hand
            .get(by)
            .unwrap()
            .iter()
            .position(|x| x.clone() == card)
            .expect("Non trovo la carta");
        self.in_hand.get_mut(by).unwrap().remove(card_index);
        // E la metto sul tavolo
        self.table.push((by.clone(), card.clone()));
        if self.table.len() == self.players.len() {
            // Se tutti hanno messo una carta
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
            self.won_cards[*self.player_team.get(&winner).unwrap()]
                .append(&mut self.table.iter().map(|x| x.1.clone()).collect());
            self.table.clear(); // Just in case...
            if self.deck.len() >= self.players.len() {
                for i in 0..self.players.len() {
                    let receiving_player_position =
                        (i + self.players.iter().position(|x| x == &winner).unwrap())
                            % self.players.len();
                    self.in_hand
                        .get_mut(&self.players[receiving_player_position])
                        .unwrap()
                        .push(self.deck.pop().unwrap());
                }
            }
            self.next_player = Some(winner.clone());
            let game_ended = self.in_hand.iter().map(|x| x.1.len()).max().unwrap() == 0;
            let mut res = vec![
                GameStatus::CardPlayed(by.clone(), card),
                GameStatus::WaitingForChoice(
                    winner.clone(),
                    self.in_hand.get(&winner).unwrap().clone(),
                ),
                GameStatus::RoundWon(winner),
            ];
            if game_ended {
                res.push(GameStatus::GameEnded);
            }
            res
        } else {
            self.next_player = Some(next_player.clone());
            vec![
                GameStatus::WaitingForChoice(
                    next_player.clone(),
                    self.in_hand.get(&next_player).unwrap().clone(),
                ),
                GameStatus::InProgress(next_player),
            ]
        }
    }
    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str> {
        if self.deck.len() < 40 {
            // La partita è gia cominciata, errore!
            return Err("Non è possibile aggiungersi ad una partita già cominciata!");
        }
        if self.players.len() <= self.get_num_players().end as usize {
            if self.players.is_empty() {
                self.next_player = Some(player.clone());
            }
            // Aggiungo il giocatore
            self.players.push(player.clone());
            let is_ready = self.players.len() <= self.get_num_players().end as usize
                && self.get_num_players().start as usize <= self.players.len();
            let hand = Vec::new();
            self.in_hand.insert(player.clone(), hand);
            // Lo assegno ad un team
            self.teams[self.players.len() % 2].push(player.clone());
            self.player_team
                .insert(player.clone(), self.players.len() % 2);
            Ok(GameStatus::WaitingForPlayers(is_ready, player))
        } else {
            Err("Il gioco è pieno")
        }
    }
    fn get_next_player(&self) -> Option<Player> {
        self.next_player.clone()
    }
    fn start(&mut self) -> GameStatus {
        if self.started {
            return GameStatus::InvalidMove("Il gioco è già iniziato, non puoi farlo reiniziare!");
        }
        if self.players.len() == 3 {
            // FIXME controllare che non sia di briscola
            // Spostare il terzo giocatore in un team a se stante e togliere una carta
            if let Some(i) = self.deck.iter().position(|x| x.0 == CardType::Numeric(2)) {
                self.deck.remove(i);
            } else {
                // Terminiamo in anticipo il gioco, ma non dovrebbe mai succedere
                // Ma piutòst che gnit, l'è mej piutòst
                return GameStatus::GameEnded;
            }
            self.teams.push(Vec::new());
            self.won_cards.push(Vec::new());
            let moved_player = self.teams[1].pop().unwrap();
            self.teams[2].push(moved_player);
            self.player_team.remove(&self.teams[2][0].clone());
            self.player_team.insert(self.teams[2][0].clone(), 2);
        }
        // Scelgo la briscola
        self.briscola = self.deck.first().unwrap().1.clone();
        // Do le carte
        for player in &self.players {
            for _ in 0..3 {
                self.in_hand
                    .get_mut(player)
                    .unwrap()
                    .push(self.deck.pop().unwrap());
            }
        }
        let player = self.players[0].clone();
        self.started = true;
        GameStatus::WaitingForChoice(player.clone(), self.in_hand.get(&player).unwrap().clone())
    }
    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::GenericFraction<u8>)> {
        self.teams
            .iter()
            .zip(self.won_cards.iter())
            .map(|x| {
                let player_lst = x.0.clone();
                let score = x.1.iter().map(|x| Self::get_card_rank(&x.0)).sum();
                (player_lst, score)
            })
            .collect()
    }
    fn get_status(&self) -> String {
        format!(
            "Partita di {}\nPunteggi:\n{}\nBriscola è: {}\nTocca a: {}\nCarte sul tavolo:\n{}",
            self.get_name(),
            self.get_scores()
                .iter()
                .enumerate()
                .map(|x| format!(
                    "{}: {} punti",
                    (x.1).0.iter().map(|y| y.name.clone()).join(", "),
                    (x.1).1
                ))
                .join("\n"),
            String::from(&self.briscola),
            self.get_next_player()
                .map(|x| x.name)
                .unwrap_or_else(|| "".to_owned()),
            self.table
                .iter()
                .map(|x| format!("- {} ({})", utils::get_card_name(&x.1), x.0.name))
                .join("\n")
        )
    }
    fn get_players(&self) -> Vec<Player> {
        self.players.clone()
    }
    fn get_new_instance(&self) -> Box<dyn Game> {
        Box::new(Self::default())
    }
    fn handle_message(&self, message: String, from: Player) -> Vec<GameStatus> {
        vec![GameStatus::NotifyRoom(format!(
            "{} ha detto: {}",
            from.name, message
        ))]
    }
}

impl Default for Briscola {
    fn default() -> Self {
        let mut teams = Vec::new();
        teams.push(Vec::new());
        teams.push(Vec::new());
        let mut wc = Vec::new();
        wc.push(Vec::new());
        wc.push(Vec::new());
        Self {
            table: Vec::new(),
            players: Vec::new(),
            in_hand: HashMap::new(),
            teams,
            player_team: HashMap::new(),
            won_cards: wc,
            deck: vec![],
            briscola: CardSuit::Coppe,
            next_player: None,
            started: false,
        }
    }
}
