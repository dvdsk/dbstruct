use structdb::structdb;

#[structdb]
struct State {
    position: u32,
    feed: String,
}

fn main() {
    
    let state = State::new();
    dbg!(&state);
    state.set_position(5);
    // assert_eq!(state.get_position(5), 5);
}
