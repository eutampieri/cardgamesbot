use cardgames::primitives::*;
pub enum ThreadMessage {
    HandleMove(Player, Card),
    AddPlayer(Player),
    Start,
    Kill,
    Ping,
    AboutToKill,
    HandleStringMessage(Player, String)
}