use crate::primitives::*;
use crate::utils;
use itertools::Itertools;

#[derive(Debug)]
pub struct Beccaccino {
    players: Vec<Player>,
    in_hand: Vec<Vec<Card>>,
    briscola: Option<CardSuit>,
    table: Vec<(Player, Card)>,
    won_cards: Vec<(Vec<Card>, bool)>,
    next_player: Option<usize>,
}

impl Beccaccino {
    /// WHo's got the 4 of denara? Well, he's to choose the briscola!!
    fn get_choosing_player(&self) -> usize {
        self.in_hand
            .iter()
            .position(|x| {
                x.iter()
                    .any(|y| y == &(CardType::Numeric(4), CardSuit::Denari))
            })
            .unwrap()
    }
}

impl Game for Beccaccino {
    fn get_num_players(&self) -> std::ops::Range<u8> {
        4..4
    }
    fn get_card_set(&self) -> CardDeckType {
        CardDeckType::Briscola
    }
    fn get_name(&self) -> &str {
        "Beccaccino"
    }
    fn init(&mut self) {}
    fn add_player(&mut self, player: Player) -> Result<GameStatus, &str> {
        if self.players.len() > 4 {
            Err("La partita è al completo")
        } else if !self.in_hand[0].is_empty() {
            Err("La partita è già cominciata")
        } else {
            self.players.push(player.clone());
            if self.players.len() == 4 {
                Ok(GameStatus::WaitingForPlayers(true, player))
            } else {
                Ok(GameStatus::WaitingForPlayers(false, player))
            }
        }
    }
    fn get_card_rank(card: &CardType) -> fraction::GenericFraction<u8> {
        match card {
            CardType::Jack => fraction::GenericFraction::new(1u8, 3u8),
            CardType::Queen => fraction::GenericFraction::new(1u8, 3u8),
            CardType::King => fraction::GenericFraction::new(1u8, 3u8),
            CardType::Numeric(x) => match x {
                1 => fraction::GenericFraction::new(1u8, 1u8),
                2 => fraction::GenericFraction::new(1u8, 3u8),
                3 => fraction::GenericFraction::new(1u8, 3u8),
                _ => fraction::GenericFraction::new(0u8, 3u8),
            },
            CardType::Jolly => fraction::GenericFraction::new(0u8, 0u8),
        }
    }
    fn get_card_sorting_rank(card: &CardType) -> u8 {
        match card {
            CardType::Jack => 5,
            CardType::Queen => 6,
            CardType::King => 7,
            CardType::Numeric(x) => match x {
                1 => 8,
                2 => 9,
                3 => 10,
                7 => 4,
                6 => 3,
                5 => 2,
                _ => 1,
            },
            CardType::Jolly => 0,
        }
    }
    fn start(&mut self) -> GameStatus {
        // Se la partita è già cominciata segnalo l'errore
        if !self.in_hand[0].is_empty() {
            return GameStatus::InvalidMove("Il gioco è già iniziato, non puoi farlo reiniziare!");
        }
        // Genero il mazzo e do le carte
        let deck = utils::random_deck(CardDeckType::Briscola);
        for i in 0..4 {
            self.in_hand[i].extend_from_slice(&deck[i * 10..(i + 1) * 10]);
        }
        // Determino chi ha la briscola
        let choosing_player = self.get_choosing_player();
        self.next_player = Some(choosing_player);
        GameStatus::WaitingForChoiceCustomMessage(
            self.players[choosing_player].clone(),
            self.in_hand[choosing_player].clone(),
            "Scegli quale sarà il seme seme di briscola",
        )
    }
    fn get_next_player(&self) -> Option<Player> {
        self.next_player.map(|x| self.players[x].clone())
    }
    fn get_scores(&self) -> Vec<(Vec<Player>, fraction::GenericFraction<u8>)> {
        vec![vec![0usize, 2], vec![1, 3]]
            .iter()
            .zip(self.won_cards.iter())
            .map(|y| {
                let player_lst = y.0.iter().map(|z| self.players[*z].clone()).collect();
                let score = {
                    if (y.1).1 {
                        utils::one()
                    } else {
                        utils::zero()
                    }
                } + (y.1)
                    .0
                    .iter()
                    .map(|x| Self::get_card_rank(&x.0))
                    .fold(fraction::GenericFraction::new(0u8, 0u8), |acc, x| acc + x);
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
            &self
                .briscola
                .clone()
                .map(|x| String::from(&x))
                .unwrap_or_else(|| "non ancora scelta".to_owned()),
            self.get_next_player()
                .map(|x| x.name)
                .unwrap_or_else(|| "".to_owned()),
            self.table
                .iter()
                .map(|x| format!("- {} ({})", utils::get_card_name(&x.1), x.0.name))
                .join("\n")
        )
    }
    fn handle_move(&mut self, by: &Player, card: Card) -> Vec<GameStatus> {
        /*
        il prinmo turno il giocatore con il 4 di denara sceglie la briscola
        e gioca una carta.
        Le carte sul tavolo devono essere dello stesso seme o, se uno le ha finite, di qualsiasi altro seme
        */
        if (&self.briscola).is_none() {
            let choosing_player = self.get_choosing_player();
            if by == &self.players[choosing_player] {
                self.briscola = Some(card.1);
                vec![GameStatus::WaitingForChoice(
                    by.clone(),
                    self.in_hand[choosing_player].clone(),
                )]
            } else {
                vec![GameStatus::InvalidMove(
                    "Non tocca a te scegliere la briscola!",
                )]
            }
        } else {
            // bloccare mossa se non è il tuo turno
            if self.get_next_player().is_none() {
                return vec![GameStatus::InvalidMove("La partita non è ancora iniziata")];
            } else if &self.get_next_player().unwrap() != by {
                return vec![GameStatus::InvalidMove("Non è ancora il tuo turno!")];
            }
            let player_index = self.players.iter().position(|x| x == by).unwrap();
            let next_player_index = (player_index + 1) % 4;
            if self.table.is_empty() {
                // è la prima carta, salto le limitazioni del seme
                let card_index = self.in_hand[player_index]
                    .iter()
                    .position(|x| x.clone() == card)
                    .expect("Non trovo la carta");
                self.in_hand[player_index].remove(card_index);
                self.table.push((by.clone(), card));
                self.next_player = Some(next_player_index);
            } else if self.table.len() < 4 {
                // Controllo il seme
                if card.1 == (self.table[0].1).1 {
                    // Il seme è giusto, aggiungo
                    let card_index = self.in_hand[player_index]
                        .iter()
                        .position(|x| x.clone() == card)
                        .expect("Non trovo la carta");
                    self.in_hand[player_index].remove(card_index);
                    self.table.push((by.clone(), card));
                    self.next_player = Some(next_player_index);
                //vec![GameStatus::WaitingForChoice(self.players[next_player_index].clone(), self.in_hand[next_player_index].clone())]
                } else {
                    let first_suit = &(self.table[0].1).1;
                    if self.in_hand[player_index]
                        .iter()
                        .any(|x| &(x.1) == first_suit)
                    {
                        // Sta barando, fermiamolo!
                        return vec![
                            GameStatus::InvalidMove(
                                "Devi giocare una carta dello stesso seme della prima!",
                            ),
                            GameStatus::WaitingForChoice(
                                by.clone(),
                                self.in_hand[player_index].clone(),
                            ),
                        ];
                    } else {
                        let card_index = self.in_hand[player_index]
                            .iter()
                            .position(|x| x.clone() == card)
                            .expect("Non trovo la carta");
                        self.in_hand[player_index].remove(card_index);
                        self.next_player = Some(next_player_index);
                        //vec![GameStatus::WaitingForChoice(self.players[next_player_index].clone(), self.in_hand[next_player_index].clone())]
                    }
                }
            }
            if self.table.len() == 4 {
                // Se è il tavolo è pieno
                // Calcolo il vincitore
                let mut winner = (self.table[0]).clone().0;
                let mut winning_suit = (&(self.table[0]).1).clone().1;
                let mut max: i32 = -1;
                for (player, card) in &self.table {
                    if card.1 == self.briscola.clone().unwrap()
                        && winning_suit != self.briscola.clone().unwrap()
                    {
                        let temp_suit = self.briscola.clone().unwrap();
                        winning_suit = temp_suit;
                        max = Self::get_card_sorting_rank(&card.0) as i32;
                        winner = player.clone();
                    }
                    if card.1 == winning_suit && Self::get_card_sorting_rank(&card.0) as i32 > max {
                        max = Self::get_card_sorting_rank(&card.0) as i32;
                        winner = player.clone();
                    }
                }
                // Abbiamo determinato chi ha vinto la mano, assegnamogliela
                let winner_index = self.players.iter().position(|x| x == &winner).unwrap();
                let winner_team_index = winner_index % 2;
                self.won_cards[winner_team_index]
                    .0
                    .append(&mut self.table.iter().map(|x| x.1.clone()).collect());
                self.table.clear(); // Just in case...
                if self.in_hand.iter().map(|x| x.len()).max().unwrap() == 0 {
                    self.won_cards[winner_team_index].1 = true;
                    return vec![
                        GameStatus::RoundWon(self.players[winner_index].clone()),
                        GameStatus::GameEnded,
                        GameStatus::NotifyRoom(self.get_status()),
                    ];
                } else {
                    self.next_player = Some(winner_index);
                }
            }
            vec![
                GameStatus::WaitingForChoice(
                    self.players[next_player_index].clone(),
                    self.in_hand[next_player_index].clone(),
                ),
                GameStatus::NotifyUser(self.players[next_player_index].clone(), self.get_status()),
            ]
        }
    }
    fn get_players(&self) -> Vec<Player> {
        self.players.clone()
    }
    fn get_new_instance(&self) -> Box<dyn Game> {
        Box::new(Self::default())
    }
    fn handle_message(&self, message: String, from: Player) -> Vec<GameStatus> {
        // Check if the message is valid (striscio, busso or volo)
        match message
            .to_lowercase()
            .as_str()
            .chars()
            .filter(|x| x.is_alphabetic())
            .collect::<String>()
            .as_str()
        {
            "busso" => vec![GameStatus::NotifyRoom(format!(
                "{} ha detto: Busso",
                from.name
            ))],
            "striscio" => vec![GameStatus::NotifyRoom(format!(
                "{} ha detto: Striscio",
                from.name
            ))],
            "volo" => vec![GameStatus::NotifyRoom(format!(
                "{} ha detto: Volo",
                from.name
            ))],
            _ => vec![GameStatus::NotifyUser(
                from,
                "Puoi dire solo busso, striscio o volo.".to_owned(),
            )],
        }
    }
}

impl Default for Beccaccino {
    fn default() -> Self {
        Self {
            players: vec![],
            in_hand: vec![vec![], vec![], vec![], vec![]],
            briscola: None,
            table: vec![],
            won_cards: vec![(vec![], false), (vec![], false)],
            next_player: None,
        }
    }
}
