use enum_map::EnumMap;
use enumset::EnumSet;

use crate::bot::behavior::freestyle::evaluator::evaluate;
use crate::bot::behavior::freestyle::score::Eval;
use crate::bot::behavior::{Behavior, BehaviorSwitch};
use crate::bot::{BotOptions, Statistics};
use crate::search::{ChildData, Dag, SearchContext};
use crate::tetris::model::{BoardRepresentation, GameState, Piece, Placement};
use crate::tetris::movegen::find_moves;
use crate::tetris::movegen::MovegenBoard;

pub struct Freestyle<B: BoardRepresentation> {
    dag: Dag<Eval, B>,
}

impl<B: BoardRepresentation> Freestyle<B> {
    pub fn new(options: &BotOptions, root: GameState<B>, queue: &[Piece]) -> Self {
        let locking = options.config.search.budget.starts_worker();
        let shard_count = if locking {
            // Search touches several layers for every expansion. Contention
            // testing found that a high shard density pays for its modest
            // per-layer memory cost by keeping unrelated states independent.
            let workers = options.config.search.threads.get();
            if workers == 1 {
                1
            } else {
                workers.saturating_mul(16).next_power_of_two()
            }
        } else {
            1
        };
        Freestyle {
            dag: Dag::new(root, queue, locking, shard_count),
        }
    }
}

impl<B: MovegenBoard> Behavior<B> for Freestyle<B> {
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

    fn step_search(&self, options: &BotOptions, context: &SearchContext) -> Statistics {
        puffin::profile_function!();
        let mut new_stats = Statistics::default();
        new_stats.selections += 1;

        if let Some(node) = self.dag.select(
            options.speculate,
            options.config.behaviors.freestyle.exploitation,
            &options.rules,
            context,
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
                        let mut state = state.clone();
                        let info = state.advance(next, mv, &options.rules);

                        let (eval, reward) = evaluate(
                            &options.config.behaviors.freestyle.weights,
                            state.clone(),
                            &info,
                            sd_distance,
                        );

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
