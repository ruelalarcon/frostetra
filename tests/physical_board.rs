use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use frostetra::config::{BotConfig, SearchBudgetConfig, SearchRngConfig};
use frostetra::tetris::model::{
    Board, BoardRepresentation, Piece, PieceLocation, Placement, Rotation,
};
use futures::{sink, stream};
use serde_json::{json, Value};

fn suggestions(messages: Vec<Value>) -> Vec<Placement> {
    let mut config = BotConfig::default();
    config.search.budget = SearchBudgetConfig::IterationsPerSuggest { iterations: 100 };
    config.search.rng = SearchRngConfig::Seeded { seed: 42 };
    let responses = Arc::new(Mutex::new(Vec::new()));
    let captured = responses.clone();
    let outgoing = sink::unfold((), move |(), message| {
        captured
            .lock()
            .unwrap()
            .push(serde_json::to_value(message).unwrap());
        async { Ok::<_, Infallible>(()) }
    });
    futures::pin_mut!(outgoing);
    let incoming = stream::iter(
        messages
            .into_iter()
            .map(|m| serde_json::from_value(m).unwrap()),
    );
    futures::executor::block_on(frostetra::run(incoming, outgoing, Arc::new(config)));
    let responses = responses.lock().unwrap();
    responses
        .iter()
        .filter(|m| m["type"] == "suggestion")
        .map(|m| serde_json::from_value(m["moves"][0].clone()).expect("nonempty suggestion"))
        .collect()
}

#[test]
fn pending_garbage_does_not_raise_the_board_on_advance() {
    for pending in [json!([]), json!([5])] {
        let locked = PieceLocation {
            piece: Piece::O,
            rotation: Rotation::North,
            x: 4,
            y: 0,
        };
        let mut physical = Board::default();
        physical.place(locked);
        let moves = suggestions(vec![
            json!({"type": "start", "board": vec![vec![Value::Null; 10]; 40],
                "active": "O", "hold": "O", "queue": ["T", "J", "L", "S", "Z"],
                "combo": 0, "back_to_back": 0, "incoming_garbage": pending}),
            json!({"type": "advance", "move": {"location": locked, "spin": "none"}}),
            json!({"type": "suggest", "incoming_garbage": pending}),
            json!({"type": "stop"}),
        ]);
        assert_eq!(moves.len(), 1);
        assert!(!moves[0].location.obstructed(&physical));
        assert_eq!(
            moves[0].location.drop_distance(&physical),
            0,
            "{pending}: {:?}",
            moves[0]
        );
    }
}

#[test]
fn board_message_confirms_the_actual_garbage_rise() {
    let locked = PieceLocation {
        piece: Piece::O,
        rotation: Rotation::North,
        x: 4,
        y: 0,
    };
    let mut rows = vec![vec![Value::Null; 10]; 40];
    let mut physical = Board::default();
    for y in 0..5 {
        for x in 0..10 {
            if x != 7 {
                rows[y][x] = json!("G");
                physical.cols[x] |= 1 << y;
            }
        }
    }
    for (x, y) in locked.cells() {
        rows[y as usize + 5][x as usize] = json!("O");
        physical.cols[x as usize] |= 1 << (y + 5);
    }
    let moves = suggestions(vec![
        json!({"type": "start", "board": vec![vec![Value::Null; 10]; 40],
            "active": "O", "hold": "O", "queue": ["T", "J", "L", "S", "Z"],
            "combo": 0, "back_to_back": 0, "incoming_garbage": [5]}),
        json!({"type": "advance", "move": {"location": locked, "spin": "none"}}),
        json!({"type": "board", "board": rows}),
        json!({"type": "suggest", "incoming_garbage": []}),
        json!({"type": "stop"}),
    ]);
    assert_eq!(moves.len(), 1);
    assert!(!moves[0].location.obstructed(&physical));
    assert_eq!(moves[0].location.drop_distance(&physical), 0);
}
