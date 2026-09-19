import random

import pytest
from fastapi.testclient import TestClient

from server.app import create_app

SETUP_MOVES = {f"MOVE {file}1 {file}3" for file in "ABCDEFG"}


@pytest.fixture
def db_path(tmp_path):
    return tmp_path / "games.sqlite"


@pytest.fixture
def client(db_path):
    return TestClient(create_app(db_path, rng=random.Random(0)))


def new_game(client, mode="hotseat"):
    response = client.post("/api/games", json={"mode": mode})
    assert response.status_code == 201
    return response.json()


def play(client, game, action):
    return client.post(
        f"/api/games/{game['id']}/actions",
        json={"action": action, "version": game["version"]},
    )


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
    assert game["human_players"] == ["First"]

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


def test_games_survive_a_restart(db_path):
    first_server = TestClient(create_app(db_path, rng=random.Random(0)))
    game = new_game(first_server, mode="bot")
    game = play(first_server, game, "MOVE D1 D3").json()

    second_server = TestClient(create_app(db_path, rng=random.Random(0)))
    restored = second_server.get(f"/api/games/{game['id']}").json()

    assert restored == game


def test_games_are_listed_most_recent_first(client):
    older = new_game(client)
    newer = new_game(client, mode="bot")
    play(client, newer, "MOVE D1 D3")

    summaries = client.get("/api/games").json()

    assert [summary["id"] for summary in summaries] == [newer["id"], older["id"]]
    assert summaries[0]["mode"] == "bot"
    assert summaries[0]["version"] == 2


def test_the_browser_client_is_served(client):
    response = client.get("/")

    assert response.status_code == 200
    assert "Dus Dus Dus" in response.text
