import random

import pytest
from dus_engine import Action, GameRecord, GameState
from fastapi.testclient import TestClient

from server.app import create_app
from server.games import Game, Mode
from server.storage import GameStore

SETUP_MOVES = {f"MOVE {file}1 {file}3" for file in "ABCDEFG"}


@pytest.fixture
def db_path(tmp_path):
    return tmp_path / "games.sqlite"


@pytest.fixture
def client(db_path):
    return TestClient(create_app(db_path, rng=random.Random(0)))


JET = {"X-Player": "player-jet"}
SAM = {"X-Player": "player-sam"}


def new_game(client, mode="hotseat", name="Jet", headers=JET):
    response = client.post("/api/games", json={"mode": mode, "name": name}, headers=headers)
    assert response.status_code == 201
    return response.json()


def play(client, game, action, headers=JET):
    return client.post(
        f"/api/games/{game['id']}/actions",
        json={"action": action, "version": game["version"]},
        headers=headers,
    )


def join(client, game, name, headers):
    return client.post(
        f"/api/games/{game['id']}/join",
        json={"invite": game["invite"], "name": name},
        headers=headers,
    )


def store_record(db_path, id, record, mode=Mode.HOTSEAT, seats=None):
    """Save a record straight to the database, as an older server might have."""
    GameStore(db_path).insert(Game(id, mode, record, GameState(), seats or {}))


def finished_record(seed=0):
    rng = random.Random(seed)
    record, state = GameRecord(), GameState()
    while not state.is_terminal:
        action = rng.choice(state.valid_actions())
        record.push(action)
        state = state.apply(action)
    record.winner = state.winner
    return record


def test_a_new_game_starts_in_setup(client):
    game = new_game(client)

    assert game["version"] == 0
    assert game["state"]["setup"]
    assert game["state"]["current_player"] == "First"
    assert game["state"]["ball"] is None
    assert game["state"]["winner"] is None
    assert game["state"]["pieces"]["A1"] == "First"
    assert game["state"]["pieces"]["A7"] == "Second"
    assert set(game["valid_actions"]) == SETUP_MOVES
    assert game["history"] == []


def test_playing_an_action_advances_the_game(client):
    game = new_game(client)

    response = play(client, game, "MOVE D1 D3")

    assert response.status_code == 200
    game = response.json()
    assert game["version"] == 1
    assert game["state"]["current_player"] == "Second"
    assert game["state"]["ball"] == "D3"
    assert game["history"] == [{"player": "First", "action": "MOVE D1 D3"}]


def test_an_illegal_action_is_rejected_with_the_engines_reason(client):
    game = new_game(client)

    response = play(client, game, "MOVE A1 A2")

    assert response.status_code == 400
    assert response.json()["detail"] == "Path too short"


def test_unparseable_notation_is_rejected(client):
    game = new_game(client)

    response = play(client, game, "WAFFLE A1 A3")

    assert response.status_code == 400


def test_a_stale_version_is_rejected(client):
    game = new_game(client)
    play(client, game, "MOVE D1 D3")

    # Resubmitting against version 0, as a double click would, must not replay.
    response = play(client, game, "MOVE A1 A3")

    assert response.status_code == 409
    assert client.get(f"/api/games/{game['id']}").json()["version"] == 1


def test_an_unknown_game_is_not_found(client):
    assert client.get("/api/games/nope").status_code == 404


def test_the_bot_replies_within_the_same_request(client):
    game = new_game(client, mode="bot")
    assert game["can_play"] == ["First"]

    game = play(client, game, "MOVE D1 D3").json()

    assert game["version"] == 2
    assert game["state"]["current_player"] == "First"
    assert [entry["player"] for entry in game["history"]] == ["First", "Second"]


def test_a_bot_game_can_be_played_to_the_end(client):
    rng = random.Random(1)
    game = new_game(client, mode="bot")

    for _ in range(5000):
        if game["state"]["winner"] is not None:
            break
        response = play(client, game, rng.choice(game["valid_actions"]))
        assert response.status_code == 200, response.json()
        game = response.json()

    assert game["state"]["winner"] in ("First", "Second")
    assert game["valid_actions"] == []
    assert game["version"] == len(game["history"])

    response = play(client, game, "MOVE A1 A3")
    assert response.status_code == 400
    assert response.json()["detail"] == "Game is over"


def test_a_game_records_who_is_playing_it(client):
    game = new_game(client, mode="bot", name="Jet")

    assert game["seats"]["First"] == {"name": "Jet", "is_bot": False, "is_you": True}
    assert game["seats"]["Second"] == {"name": "Bot", "is_bot": True, "is_you": False}
    assert game["can_play"] == ["First"]

    # Someone else sees the same seats, but none of them as theirs to play.
    seen = client.get(f"/api/games/{game['id']}", headers=SAM).json()
    assert seen["seats"]["First"]["name"] == "Jet"
    assert seen["seats"]["First"]["is_you"] is False
    assert seen["can_play"] == []


def test_only_the_player_holding_a_seat_can_move(client):
    game = new_game(client)

    refused = play(client, game, "MOVE D1 D3", headers=SAM)

    assert refused.status_code == 403
    assert "Jet" in refused.json()["detail"]
    assert play(client, game, "MOVE D1 D3", headers=JET).status_code == 200


def test_a_hot_seat_game_lets_its_owner_play_both_sides(client):
    game = new_game(client)

    game = play(client, game, "MOVE D1 D3").json()
    assert game["state"]["current_player"] == "Second"
    assert game["can_play"] == ["First", "Second"]

    assert play(client, game, "MOVE A7 A5").status_code == 200


def test_a_game_from_before_seats_existed_is_open_to_anyone(client, db_path):
    store_record(db_path, "legacy", GameRecord(actions=[Action("MOVE D1 D3")]))

    game = client.get("/api/games/legacy", headers=SAM).json()

    assert game["seats"]["First"]["name"] is None
    assert game["can_play"] == ["First", "Second"]
    assert play(client, game, "MOVE A7 A5", headers=SAM).status_code == 200


def test_an_older_database_gains_seats_without_losing_games(db_path):
    import sqlite3

    # A database as the previous version wrote it: no seat columns, no version.
    with sqlite3.connect(db_path) as db:
        db.execute(
            "CREATE TABLE games (id TEXT PRIMARY KEY, mode TEXT NOT NULL, record TEXT NOT NULL,"
            " version INTEGER NOT NULL, winner TEXT, created_at TEXT NOT NULL,"
            " updated_at TEXT NOT NULL)"
        )
        db.execute(
            "INSERT INTO games VALUES ('old', 'hotseat', ?, 1, NULL, '2026-01-01', '2026-01-01')",
            (GameRecord(actions=[Action("MOVE D1 D3")]).to_json(),),
        )

    client = TestClient(create_app(db_path, rng=random.Random(0)))
    game = client.get("/api/games/old", headers=JET).json()

    assert game["version"] == 1
    assert game["seats"]["First"]["name"] is None
    assert [summary["id"] for summary in client.get("/api/games").json()] == ["old"]


def test_an_online_game_waits_for_someone_to_join(client):
    game = new_game(client, mode="online", name="Jet")

    mine, theirs = _taken_and_open(game)
    assert game["seats"][mine] == {"name": "Jet", "is_bot": False, "is_you": True}
    assert game["seats"][theirs] == {"name": None, "is_bot": False, "is_you": False}
    assert game["invite"]
    # Nobody may play the empty side, not even the player who is waiting.
    assert game["can_play"] == [mine]

    # The invitation is for the players, not for anyone who finds the game.
    assert client.get(f"/api/games/{game['id']}", headers=SAM).json()["invite"] is None


def test_the_sides_of_online_games_are_drawn_at_random(db_path):
    client = TestClient(create_app(db_path, rng=random.Random(7)))

    mine = {_taken_and_open(new_game(client, mode="online"))[0] for _ in range(20)}

    assert mine == {"First", "Second"}


def test_joining_takes_the_open_seat(client):
    game = new_game(client, mode="online", name="Jet")
    mine, theirs = _taken_and_open(game)

    joined = join(client, game, name="Sam", headers=SAM)

    assert joined.status_code == 200
    joined = joined.json()
    assert joined["seats"][theirs] == {"name": "Sam", "is_bot": False, "is_you": True}
    assert joined["can_play"] == [theirs]
    # The invitation is spent, and both sides are now somebody's.
    assert joined["invite"] is None
    assert client.get(f"/api/games/{game['id']}", headers=JET).json()["can_play"] == [mine]


def test_a_wrong_invitation_is_refused(client):
    game = new_game(client, mode="online")

    refused = client.post(
        f"/api/games/{game['id']}/join",
        json={"invite": "not-the-code", "name": "Sam"},
        headers=SAM,
    )

    assert refused.status_code == 403
    assert client.get(f"/api/games/{game['id']}", headers=SAM).json()["can_play"] == []


def test_a_third_player_cannot_take_a_taken_seat(client):
    game = new_game(client, mode="online")
    join(client, game, name="Sam", headers=SAM)

    response = client.post(
        f"/api/games/{game['id']}/join",
        json={"invite": game["invite"], "name": "Nosy"},
        headers={"X-Player": "player-nosy"},
    )

    assert response.status_code == 403


def test_reopening_an_invitation_is_harmless(client):
    game = new_game(client, mode="online")
    join(client, game, name="Sam", headers=SAM)

    again = join(client, game, name="Sam", headers=SAM)

    assert again.status_code == 200
    assert len(again.json()["can_play"]) == 1


def test_online_players_can_only_move_their_own_side(client):
    game = new_game(client, mode="online", name="Jet")
    game = join(client, game, name="Sam", headers=SAM).json()

    first = JET if game["seats"]["First"]["name"] == "Jet" else SAM
    second = SAM if first is JET else JET

    assert play(client, game, "MOVE D1 D3", headers=second).status_code == 403
    game = play(client, game, "MOVE D1 D3", headers=first).json()
    assert play(client, game, "MOVE A7 A5", headers=first).status_code == 403
    assert play(client, game, "MOVE A7 A5", headers=second).status_code == 200


def _taken_and_open(game):
    """The side the game's creator took, and the one still waiting."""
    taken = [side for side, seat in game["seats"].items() if seat["is_you"]]
    open_seats = [side for side, seat in game["seats"].items() if seat["name"] is None]
    assert len(taken) == 1 and len(open_seats) == 1
    return taken[0], open_seats[0]


def test_games_survive_a_restart(db_path):
    first_server = TestClient(create_app(db_path, rng=random.Random(0)))
    game = new_game(first_server, mode="bot")
    game = play(first_server, game, "MOVE D1 D3").json()

    second_server = TestClient(create_app(db_path, rng=random.Random(0)))
    restored = second_server.get(f"/api/games/{game['id']}", headers=JET).json()

    assert restored == game
    assert restored["seats"]["First"]["name"] == "Jet"


def test_a_game_the_current_rules_forbid_is_reported_not_crashed_on(client, db_path):
    # As if saved under older rules: a one-square setup move is illegal now.
    store_record(db_path, "legacy", GameRecord(actions=[Action("MOVE A1 A2")]))

    response = client.get("/api/games/legacy")

    assert response.status_code == 409
    assert "older rules" in response.json()["detail"]
    assert "Path too short" in response.json()["detail"]
    assert "legacy" in [summary["id"] for summary in client.get("/api/games").json()]
    assert play(client, {"id": "legacy", "version": 1}, "MOVE D1 D3").status_code == 409


def test_games_are_listed_most_recent_first(client):
    older = new_game(client)
    newer = new_game(client, mode="bot")
    play(client, newer, "MOVE D1 D3")

    summaries = client.get("/api/games").json()

    assert [summary["id"] for summary in summaries] == [newer["id"], older["id"]]
    assert summaries[0]["mode"] == "bot"
    assert summaries[0]["moves"] == 2
    assert summaries[0]["seats"]["First"]["name"] == "Jet"


def test_the_list_says_how_each_game_stands(client, db_path):
    new_game(client)
    store_record(db_path, "done", finished_record())
    store_record(db_path, "legacy", GameRecord(actions=[Action("MOVE A1 A2")]))

    summaries = {summary["id"]: summary for summary in client.get("/api/games").json()}
    statuses = {summary["status"] for summary in summaries.values()}

    assert statuses == {"in_progress", "finished", "unreplayable"}
    assert summaries["done"]["winner"] in ("First", "Second")
    assert summaries["legacy"]["problem"] == "Move 1 (MOVE A1 A2) is not allowed: Path too short"


def test_a_replay_has_a_frame_for_every_move(client):
    game = new_game(client, mode="bot")
    game = play(client, game, "MOVE D1 D3").json()

    replay = client.get(f"/api/games/{game['id']}/replay").json()

    assert len(replay["frames"]) == len(game["history"]) + 1
    assert replay["frames"][0]["pieces"]["D1"] == "First"
    assert replay["frames"][0]["ball"] is None
    assert replay["frames"][-1] == game["state"]
    assert replay["history"] == game["history"]
    assert replay["failure"] is None


def test_a_replay_stops_at_a_move_the_rules_now_forbid(client, db_path):
    # The first move is fine; Teal's one-square setup move is not.
    record = GameRecord(actions=[Action("MOVE D1 D3"), Action("MOVE A7 A6")])
    store_record(db_path, "legacy", record)

    replay = client.get("/api/games/legacy/replay").json()

    assert len(replay["frames"]) == 2
    assert replay["history"] == [{"player": "First", "action": "MOVE D1 D3"}]
    assert replay["failure"] == {
        "move": 2,
        "player": "Second",
        "action": "MOVE A7 A6",
        "reason": "Path too short",
    }


def test_replaying_an_unknown_game_is_not_found(client):
    assert client.get("/api/games/nope/replay").status_code == 404


def test_the_browser_client_is_served(client):
    response = client.get("/")

    assert response.status_code == 200
    assert "Dus Dus Dus" in response.text


def test_the_browser_must_check_for_new_versions_of_the_client(client):
    for path in ("/", "/app.js", "/replay.js", "/style.css"):
        assert client.get(path).headers["cache-control"] == "no-cache", path
