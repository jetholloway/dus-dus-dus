import json

import pytest

from dus_engine import Action, GameRecord, GameState, InvalidAction, Player, Position


def test_position_round_trips_through_notation():
    position = Position(0, 0)
    assert (position.x, position.y) == (0, 0)
    assert str(position) == "A1"
    assert Position.parse("A1") == position
    assert Position.parse("g7") == Position(6, 6)


def test_action_parses_and_prints_notation():
    action = Action("MOVE A1 A3")
    assert str(action) == "MOVE A1 A3"
    assert action.action_type == "MOVE"
    assert action.src == Position.parse("A1")
    assert action.dst == Position.parse("A3")
    assert action == Action("MOVE A1 A3")
    assert len({Action("MOVE A1 A3"), Action("MOVE A1 A3")}) == 1


def test_action_rejects_nonsense():
    with pytest.raises(ValueError):
        Action("WAFFLE A1 A3")


def test_opening_position():
    state = GameState()
    assert state.current_player == Player.First
    assert state.setup
    assert state.turn_count == 0
    assert state.winner is None
    assert not state.is_terminal
    assert state.ball is None
    assert state.piece_at(Position.parse("A1")) == Player.First
    assert state.piece_at(Position.parse("A7")) == Player.Second
    assert state.piece_at(Position.parse("A4")) is None


def test_opening_position_has_seven_legal_actions():
    actions = GameState().valid_actions()
    assert sorted(str(action) for action in actions) == [
        f"MOVE {file}1 {file}3" for file in "ABCDEFG"
    ]


def test_apply_returns_a_new_state_and_leaves_the_old_one_alone():
    state = GameState()
    next_state = state.apply(Action("MOVE A1 A3"))

    assert next_state is not state
    assert state.current_player == Player.First
    assert next_state.current_player == Player.Second
    assert next_state.ball == Position.parse("A3")
    assert state.ball is None


def test_apply_raises_on_an_illegal_action():
    with pytest.raises(InvalidAction) as error:
        GameState().apply(Action("MOVE A1 A2"))

    assert str(error.value)


def test_state_round_trips_through_json():
    state = GameState().apply(Action("MOVE A1 A3"))

    restored = GameState.from_json(state.to_json())

    assert restored == state
    assert json.loads(state.to_json())["turn"]["player"] == "Second"


def play_random_game(seed=0):
    import random

    rng = random.Random(seed)
    state = GameState()
    record = GameRecord()

    while not state.is_terminal:
        action = rng.choice(state.valid_actions())
        record.push(action)
        state = state.apply(action)

    record.winner = state.winner
    return record, state


def test_a_whole_game_can_be_played_and_replayed():
    record, final_state = play_random_game()

    assert final_state.is_terminal
    assert final_state.winner in (Player.First, Player.Second)
    assert record.winner == final_state.winner

    states = record.replay()

    assert len(states) == len(record.actions) + 1
    assert states[-1] == final_state


def test_a_finished_game_accepts_no_more_actions():
    _, final_state = play_random_game(seed=3)

    assert final_state.valid_actions() == []
    with pytest.raises(InvalidAction):
        final_state.apply(Action("MOVE A1 A3"))


def test_record_round_trips_through_json():
    record, _ = play_random_game(seed=1)

    restored = GameRecord.from_json(record.to_json())

    assert restored == record
    assert json.loads(record.to_json())["actions"][0] == str(record.actions[0])


def test_replay_rejects_a_corrupted_record():
    record, _ = play_random_game(seed=2)
    corrupted = GameRecord(actions=[Action("MOVE A1 G7")] + record.actions[1:],
                           winner=record.winner)

    with pytest.raises(ValueError):
        corrupted.replay()
