//! # Card Games
//! This crate provide several card games that can be conveniently accessed through a shared set of primitives.
//! ## Adding your game
//! To add your game you have to:
//! - Fork [the repo](https://github.com/eutampieri/cardgamesbot)
//! - Create a new file under `games/src/games` named after the game
//! - Create a public `struct` representing your game and implementing the `Default` and the `Game` traits
//!     * Most of the methods are documented, but the main one is `handle_move` which updates the game status according to the card recieved
//! - Export your game in `games/src/games/mod.rs`
//! - Implement some tests
//! - Open a pull request on the main repo

pub mod games;
pub mod primitives;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
