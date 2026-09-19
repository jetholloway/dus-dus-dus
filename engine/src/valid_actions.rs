use super::*;

const MOVE_DISPLACEMENTS: [Displacement; 8] = [
    Displacement { x: -2, y: 0 },
    Displacement { x: -1, y: 0 },
    Displacement { x: 1, y: 0 },
    Displacement { x: 2, y: 0 },
    Displacement { x: 0, y: -2 },
    Displacement { x: 0, y: -1 },
    Displacement { x: 0, y: 1 },
    Displacement { x: 0, y: 2 },
];

impl GameState {
    pub fn valid_actions(&self) -> Vec<Action> {
        self.candidate_actions()
            .into_iter()
            .filter(|action| !matches!(action.try_apply(self), ActionResult::Invalid(_)))
            .collect()
    }

    fn candidate_actions(&self) -> Vec<Action> {
        let player = self.current_player();
        let ball = self.ball();
        let mut candidates = Vec::new();

        for src in Self::positions() {
            if self.space(src) != Space::Piece(player) {
                continue;
            }

            for displacement in MOVE_DISPLACEMENTS {
                candidates.push(Action::new_move(src, src + displacement));
            }

            if src.taxicab_distance(ball) == 1 {
                for dst in src.neighbours() {
                    candidates.push(Action::new_tackle(src, dst));
                }
            }

            if src == ball {
                for dst in Self::positions() {
                    if dst != src && self.space(dst) == Space::Piece(player) {
                        candidates.push(Action::new_pass(src, dst));
                    }
                }
            }
        }

        candidates
    }

    fn positions() -> impl Iterator<Item = Position> {
        (0..7i8).flat_map(|y| (0..7i8).map(move |x| Position { x, y }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    use std::collections::HashSet;

    const GAMES: usize = 10;
    const MAX_ACTIONS_PER_GAME: usize = 400;

    fn every_action() -> Vec<Action> {
        let mut actions = Vec::new();

        for action_type in [ActionType::Move, ActionType::Tackle, ActionType::Pass] {
            for src in GameState::positions() {
                for dst in GameState::positions() {
                    actions.push(Action::new(action_type, src, dst));
                }
            }
        }

        actions
    }

    fn valid_actions_by_exhaustive_search(state: &GameState) -> HashSet<Action> {
        every_action()
            .into_iter()
            .filter(|action| !matches!(action.try_apply(state), ActionResult::Invalid(_)))
            .collect()
    }

    #[test]
    fn opening_position_allows_each_piece_two_squares_forward() {
        let expected: HashSet<Action> = (0..7i8)
            .map(|x| Action::new_move(Position { x, y: 0 }, Position { x, y: 2 }))
            .collect();

        let actions: HashSet<Action> = GameState::new().valid_actions().into_iter().collect();

        assert_eq!(actions, expected);
    }

    #[test]
    fn valid_actions_agrees_with_exhaustive_search() {
        let mut rng = StdRng::seed_from_u64(20260918);

        for game in 0..GAMES {
            let mut state = GameState::new();

            for _ in 0..MAX_ACTIONS_PER_GAME {
                let actions = state.valid_actions();

                assert_eq!(
                    actions.iter().cloned().collect::<HashSet<Action>>(),
                    valid_actions_by_exhaustive_search(&state),
                    "game {game} disagreed on state:\n{state:?}"
                );

                let action = actions
                    .choose(&mut rng)
                    .expect("non-terminal state should have valid actions");

                match action.try_apply(&state) {
                    ActionResult::Invalid(error) => panic!("valid action rejected: {error}"),
                    ActionResult::Valid { state: next_state } => state = next_state,
                    ActionResult::Terminal { .. } => break,
                }
            }
        }
    }
}
