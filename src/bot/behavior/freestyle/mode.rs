use enum_map::EnumMap;
use enumset::EnumSet;

use crate::bot::behavior::freestyle::evaluator::evaluate;
use crate::bot::behavior::freestyle::score::Eval;
use crate::bot::behavior::{Behavior, BehaviorSwitch};
use crate::bot::{BotOptions, Statistics};
use crate::search::{ChildData, Dag, RootCandidate, SearchContext};
use crate::tetris::model::{BoardRepresentation, GameState, Piece, Placement};
use crate::tetris::movegen::find_moves;
use crate::tetris::movegen::MovegenBoard;

const INCOMING_GARBAGE_SAFE_CENTER_HEIGHT: u32 = 18;

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

    fn suggest(&self, _options: &BotOptions, incoming_garbage: Option<&[u32]>) -> Vec<Placement> {
        puffin::profile_function!();
        let incoming = match incoming_garbage {
            Some(incoming) => incoming.iter().sum(),
            None => return self.dag.suggest(),
        };
        if incoming == 0 {
            return self.dag.suggest();
        }

        pick_for_incoming(self.dag.root_candidates(), incoming)
            .map_or_else(Vec::new, |candidate| vec![candidate.mv])
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
                            center_height: center_height(&state.board),
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

fn pick_for_incoming(
    candidates: Vec<RootCandidate<Eval>>,
    incoming: u32,
) -> Option<RootCandidate<Eval>> {
    let mut backup = None;
    for candidate in candidates {
        if candidate
            .center_height
            .saturating_add(incoming.saturating_sub(candidate.reward.attack()))
            <= INCOMING_GARBAGE_SAFE_CENTER_HEIGHT
        {
            return Some(candidate);
        }

        match backup {
            None => backup = Some(candidate),
            Some(current) if current.cached_eval.spike() < candidate.cached_eval.spike() => {
                backup = Some(candidate);
            }
            _ => {}
        }
    }
    backup
}

fn center_height(board: &impl BoardRepresentation) -> u32 {
    let width = board.width();
    let start = 3.min(width);
    let end = 6.min(width);
    let cols = if start < end {
        &board.cols()[start..end]
    } else {
        board.cols()
    };
    cols.iter()
        .map(|&col| 64 - col.leading_zeros())
        .max()
        .unwrap_or(0)
}
