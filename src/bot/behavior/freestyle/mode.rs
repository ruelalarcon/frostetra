use enum_map::EnumMap;
use enumset::EnumSet;

use crate::bot::behavior::freestyle::evaluator::evaluate;
use crate::bot::behavior::freestyle::score::Eval;
use crate::bot::behavior::{Behavior, BehaviorSwitch};
use crate::bot::{BotOptions, Statistics};
use crate::search::{ChildData, Dag};
use crate::tetris::model::{GameState, Piece, Placement};
use crate::tetris::movegen::find_moves;

pub struct Freestyle {
    dag: Dag<Eval>,
}

impl Freestyle {
    pub fn new(_options: &BotOptions, root: GameState, queue: &[Piece]) -> Self {
        Freestyle {
            dag: Dag::new(root, queue),
        }
    }
}

impl Behavior for Freestyle {
    fn advance(&mut self, options: &BotOptions, mv: Placement) -> Option<BehaviorSwitch> {
        puffin::profile_function!();
        self.dag.advance(mv, &options.rules);
        None
    }

    fn new_piece(&mut self, _options: &BotOptions, piece: Piece) {
        puffin::profile_function!();
        self.dag.add_piece(piece);
    }

    fn suggest(&self, _options: &BotOptions) -> Vec<Placement> {
        puffin::profile_function!();
        self.dag.suggest()
    }

    fn do_work(&self, options: &BotOptions) -> Statistics {
        puffin::profile_function!();
        let mut new_stats = Statistics::default();
        new_stats.selections += 1;

        if let Some(node) = self.dag.select(
            options.speculate,
            options.config.freestyle_exploitation,
            &options.rules,
        ) {
            new_stats.max_depth = node.depth();
            let (state, next) = node.state();
            let next_possibilities = next.map(EnumSet::only).unwrap_or(state.bag);

            let mut moves = EnumMap::default();
            {
                puffin::profile_scope!("movegen");
                for piece in next_possibilities | state.reserve {
                    moves[piece] = find_moves(&state.board, piece, &options.rules);
                }
            }

            let mut children: EnumMap<_, Vec<_>> = EnumMap::default();

            {
                puffin::profile_scope!("eval");
                for next in next_possibilities {
                    let moves = moves[next].iter().chain(if next == state.reserve {
                        [].iter()
                    } else {
                        moves[state.reserve].iter()
                    });
                    for &(mv, sd_distance) in moves {
                        let mut state = state;
                        let info = state.advance(next, mv, &options.rules);

                        let (eval, reward) =
                            evaluate(&options.config.weights, state, &info, sd_distance);

                        children[next].push(ChildData {
                            resulting_state: state,
                            mv,
                            eval,
                            reward,
                        });
                    }

                    new_stats.nodes += children[next].len() as u64;
                }
            }

            new_stats.expansions += 1;
            node.expand(children);
        }

        new_stats
    }
}
