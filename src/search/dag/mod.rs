use bumpalo_herd::Herd;
use enum_map::EnumMap;
use once_cell::sync::OnceCell;
use ouroboros::self_referencing;

use crate::search::SearchContext;
use crate::tetris::model::rules::GameRules;
use crate::tetris::model::Placement;
use crate::tetris::model::{GameState, Piece};

mod known;
mod speculated;

pub trait Evaluation:
    Ord + Copy + Default + std::ops::Add<Self::Reward, Output = Self> + 'static
{
    type Reward: Copy;

    fn average(of: impl Iterator<Item = Option<Self>>) -> Self;
}

pub struct Dag<E: Evaluation> {
    root: GameState,
    top_layer: Box<LayerCommon<E>>,
}

pub struct Selection<'a, E: Evaluation> {
    layers: Vec<&'a LayerCommon<E>>,
    game_state: GameState,
}

pub struct ChildData<E: Evaluation> {
    pub resulting_state: GameState,
    pub mv: Placement,
    pub eval: E,
    pub reward: E::Reward,
}

pub(super) struct LayerCommon<E: Evaluation> {
    next_layer: OnceCell<Box<LayerCommon<E>>>,
    kind: WithBump<E>,
    locking: bool,
}

#[self_referencing]
struct WithBump<E: Evaluation> {
    bump: Herd,
    #[borrows(bump)]
    #[not_covariant]
    data: LayerKind<'this, E>,
}

enum LayerKind<'bump, E: Evaluation> {
    Known(known::Layer<'bump, E>),
    Speculated(speculated::Layer<'bump, E>),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Child<E: Evaluation> {
    pub mv: Placement,
    pub reward: E::Reward,
    pub cached_eval: E,
}

pub(super) enum SelectResult {
    Failed,
    Done,
    Advance(Piece, Placement),
}

pub(super) struct BackpropUpdate {
    pub parent: u64,
    pub speculation_piece: Piece,
    pub mv: Placement,
    pub child: u64,
}

#[derive(Clone, Copy)]
pub(super) struct Parent {
    pub parent: u64,
    pub mv: Placement,
    pub speculation_piece: Piece,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Parents<'bump> {
    head: Option<&'bump ParentLink<'bump>>,
    len: usize,
}

struct ParentLink<'bump> {
    data: Parent,
    next: Option<&'bump ParentLink<'bump>>,
}

impl<'bump> Parents<'bump> {
    pub fn push(&mut self, bump: &bumpalo_herd::Member<'bump>, data: Parent) {
        let link = bump.alloc(ParentLink {
            data,
            next: self.head,
        });
        self.head = Some(link);
        self.len += 1;
    }

    pub fn for_each_ordered(self, mut f: impl FnMut(Parent)) {
        let mut parents = Vec::with_capacity(self.len);
        let mut current = self.head;
        while let Some(link) = current {
            parents.push(link.data);
            current = link.next;
        }
        for parent in parents.into_iter().rev() {
            f(parent);
        }
    }
}

impl<E: Evaluation> Dag<E> {
    pub fn new(root: GameState, queue: &[Piece], locking: bool) -> Self {
        let mut top_layer = LayerCommon::new(locking);
        top_layer.kind.initialize_root(&root);

        let mut layer = &mut top_layer;
        for &piece in queue {
            layer.kind.despeculate(piece);
            layer = layer.force_next_layer_mut();
        }

        Dag {
            root,
            top_layer: Box::new(top_layer),
        }
    }

    pub fn advance(&mut self, mv: Placement, rules: &GameRules) {
        puffin::profile_function!();
        let top_layer = std::mem::take(&mut *self.top_layer);
        self.root.advance(
            top_layer
                .kind
                .piece()
                .expect("cannot advance without next piece"),
            mv,
            rules,
        );
        top_layer.force_next_layer();
        self.top_layer = top_layer.next_layer.into_inner().unwrap();
        self.top_layer.kind.initialize_root(&self.root);
    }

    pub fn add_piece(&mut self, piece: Piece) {
        puffin::profile_function!();
        let mut layer = &mut *self.top_layer;
        loop {
            if layer.kind.despeculate(piece) {
                // TODO: backprop despeculated values
                return;
            }
            layer = layer.force_next_layer_mut();
        }
    }

    pub fn suggest(&self) -> Vec<Placement> {
        puffin::profile_function!();
        self.top_layer.kind.suggest(&self.root)
    }

    pub fn select(
        &self,
        speculate: bool,
        exploration: f64,
        rules: &GameRules,
        context: &SearchContext,
    ) -> Option<Selection<'_, E>> {
        puffin::profile_function!();
        let mut layers = vec![&*self.top_layer];
        let mut game_state = self.root;
        loop {
            let &layer = layers.last().unwrap();

            match layer
                .kind
                .select(&game_state, speculate, exploration, context)
            {
                SelectResult::Failed => return None,
                SelectResult::Done => return Some(Selection { layers, game_state }),
                SelectResult::Advance(next, placement) => {
                    game_state.advance(next, placement, rules);
                    layers.push(layer.force_next_layer());
                }
            }
        }
    }
}

impl<E: Evaluation> Selection<'_, E> {
    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    pub fn state(&self) -> (GameState, Option<Piece>) {
        (self.game_state, self.layers.last().unwrap().kind.piece())
    }

    pub fn expand(self, children: EnumMap<Piece, Vec<ChildData<E>>>) {
        puffin::profile_function!();
        let mut layers = self.layers;
        let start_layer = layers.pop().unwrap();
        let mut next =
            start_layer
                .kind
                .expand(start_layer.force_next_layer(), self.game_state, children);

        puffin::profile_scope!("backprop");
        let mut next_layer = start_layer;
        while let Some(layer) = layers.pop() {
            next = layer.kind.backprop(next, next_layer);
            next_layer = layer;

            if next.is_empty() {
                break;
            }
        }
    }
}

pub(super) fn update_child<E: Evaluation>(
    list: &mut [Child<E>],
    placement: Placement,
    child_eval: E,
) -> bool {
    let mut index = list
        .iter()
        .enumerate()
        .find_map(|(i, c)| (c.mv == placement).then(|| i))
        .unwrap();

    list[index].cached_eval = child_eval + list[index].reward;

    if index > 0 && child_precedes(&list[index], &list[index - 1]) {
        let hole = list[index];
        while index > 0 && child_precedes(&hole, &list[index - 1]) {
            list[index] = list[index - 1];
            index -= 1;
        }
        list[index] = hole;
    } else if index < list.len() - 1 && child_precedes(&list[index + 1], &list[index]) {
        let hole = list[index];
        while index < list.len() - 1 && child_precedes(&list[index + 1], &hole) {
            list[index] = list[index + 1];
            index += 1;
        }
        list[index] = hole;
    }

    index == 0
}

fn child_precedes<E: Evaluation>(left: &Child<E>, right: &Child<E>) -> bool {
    left.cached_eval > right.cached_eval
        || (left.cached_eval == right.cached_eval && left.mv.sort_key() < right.mv.sort_key())
}

impl<E: Evaluation> LayerCommon<E> {
    fn new(locking: bool) -> Self {
        LayerCommon {
            next_layer: OnceCell::new(),
            kind: WithBump::new_with_locking(locking),
            locking,
        }
    }

    fn force_next_layer(&self) -> &LayerCommon<E> {
        self.next_layer
            .get_or_init(|| Box::new(LayerCommon::new(self.locking)))
    }

    fn force_next_layer_mut(&mut self) -> &mut LayerCommon<E> {
        if self.next_layer.get().is_none() {
            let _ = self
                .next_layer
                .set(Box::new(LayerCommon::new(self.locking)));
        }
        self.next_layer.get_mut().unwrap()
    }
}

impl<E: Evaluation> Default for LayerCommon<E> {
    fn default() -> Self {
        LayerCommon::new(true)
    }
}

impl<E: Evaluation> WithBump<E> {
    fn initialize_root(&self, root: &GameState) {
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.initialize_root(root),
            LayerKind::Speculated(l) => l.initialize_root(root),
        });
    }

    fn backprop(
        &self,
        to_update: Vec<BackpropUpdate>,
        next_layer: &LayerCommon<E>,
    ) -> Vec<BackpropUpdate> {
        puffin::profile_function!();
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.backprop(to_update, next_layer),
            LayerKind::Speculated(l) => l.backprop(to_update, next_layer),
        })
    }

    fn piece(&self) -> Option<Piece> {
        self.with(|this| match this.data {
            LayerKind::Known(l) => Some(l.piece),
            LayerKind::Speculated(_) => None,
        })
    }

    fn expand(
        &self,
        next_layer: &LayerCommon<E>,
        parent_state: GameState,
        children: EnumMap<Piece, Vec<ChildData<E>>>,
    ) -> Vec<BackpropUpdate> {
        puffin::profile_function!();
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.expand(this.bump, next_layer, parent_state, children),
            LayerKind::Speculated(l) => l.expand(this.bump, next_layer, parent_state, children),
        })
    }

    fn select(
        &self,
        game_state: &GameState,
        speculate: bool,
        exploration: f64,
        context: &SearchContext,
    ) -> SelectResult {
        puffin::profile_function!();
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.select(game_state, exploration, context),
            LayerKind::Speculated(l) if speculate => l.select(game_state, exploration, context),
            LayerKind::Speculated(_) => SelectResult::Failed,
        })
    }

    fn suggest(&self, state: &GameState) -> Vec<Placement> {
        puffin::profile_function!();
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.suggest(state),
            LayerKind::Speculated(l) => l.suggest(state),
        })
    }

    fn despeculate(&mut self, piece: Piece) -> bool {
        puffin::profile_function!();
        self.with_mut(|this| {
            let old = match this.data {
                LayerKind::Known(_) => return false,
                LayerKind::Speculated(l) => std::mem::take(l),
            };

            let layer = known::Layer {
                states: old.states.map_values(|node| known::Node {
                    parents: node.parents,
                    eval: node.eval,
                    children: node.children.map(|v| v.into_children(piece)),
                    expanding: node.expanding,
                }),
                piece,
            };

            *this.data = LayerKind::Known(layer);

            true
        })
    }

    fn get_eval(&self, raw: u64) -> E {
        self.with(|this| match this.data {
            LayerKind::Known(l) => l.get_eval(raw),
            LayerKind::Speculated(l) => l.get_eval(raw),
        })
    }

    fn create_nodes(
        &self,
        children: &[ChildData<E>],
        parent: u64,
        speculation_piece: Piece,
        mut f: impl FnMut(&ChildData<E>, E),
    ) {
        self.with(|this| match this.data {
            LayerKind::Known(l) => {
                let bump = this.bump.get();
                for child in children {
                    let eval = l.create_node(&bump, child, parent, speculation_piece);
                    f(child, eval);
                }
            }
            LayerKind::Speculated(l) => {
                let bump = this.bump.get();
                for child in children {
                    let eval = l.create_node(&bump, child, parent, speculation_piece);
                    f(child, eval);
                }
            }
        })
    }
}

impl<E: Evaluation> Default for WithBump<E> {
    fn default() -> Self {
        Self::new_with_locking(true)
    }
}

impl<E: Evaluation> WithBump<E> {
    fn new_with_locking(locking: bool) -> Self {
        WithBump::new(Herd::new(), |_| {
            LayerKind::Speculated(speculated::Layer::new(locking))
        })
    }
}
