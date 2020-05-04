mod primitives;
mod briscola;
use primitives::Game;
use text_io::read;

fn main() {
    println!("Hello, world!");
    let mut b = briscola::Briscola::init();
    println!("{:?}", b);
    let p1 = primitives::Player{id: 1, name: "Player 1".to_owned()};
    let p2 = primitives::Player{id: 2, name: "Player 2".to_owned()};
    println!("{:?}", b.add_player(p1.clone()));
    println!("{:?}", b.add_player(p2.clone()));
    let mut game_status = match b.start() {
        primitives::GameStatus::WaitingForChoice(x, y) => (x, y),
        _ => {unimplemented!()}
    };
    loop {
        println!("{}. Quale carta metti? {:?}", b.get_status(), game_status.1);
        let index: usize = read!();
        let card = game_status.1.clone()[index].clone();
        let outcome = b.handle_move(&game_status.0, card);
        println!("{:?}", outcome);
        if let Some(_) = outcome.iter().find(|x| match x {primitives::GameStatus::GameEnded => true, _ => false}) {
            break;
        }
        if let Some(primitives::GameStatus::WaitingForChoice(x, y)) = outcome.iter().find(|x| match x {primitives::GameStatus::WaitingForChoice(_, _) => true, _ => false}) {
            game_status = (x.clone(), y.clone());
        }
    }
}
