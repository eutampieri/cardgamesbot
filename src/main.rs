mod primitives;
mod briscola;
mod beccaccino;
mod utils;
use primitives::Game;
use text_io::read;

fn main() {
    println!("Hello, world!");
    let mut b = beccaccino::Beccaccino::init();
    println!("{:?}", b);
    let p1 = primitives::Player{id: 1, name: "Anna".to_owned()};
    let p2 = primitives::Player{id: 2, name: "Bob".to_owned()};
    let p3 = primitives::Player{id: 3, name: "Carla".to_owned()};
    let p4 = primitives::Player{id: 4, name: "Daniele".to_owned()};
    println!("{:?}", b.add_player(p1.clone()));
    println!("{:?}", b.add_player(p2.clone()));
    println!("{:?}", b.add_player(p3.clone()));
    println!("{:?}", b.add_player(p4.clone()));
    let mut game_status = match b.start() {
        primitives::GameStatus::WaitingForChoiceCustomMessage(x, y, z) => (x, y),
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
