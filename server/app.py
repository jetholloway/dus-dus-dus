"""The web server: a JSON API over the engine, plus the static browser client.

Run it with `just serve`, which starts uvicorn on this module's create_app.
"""

import random
import secrets
from pathlib import Path

from dus_engine import GameState, Position
from fastapi import FastAPI, HTTPException
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from .games import (
    PLAYER_NAMES,
    Game,
    Mode,
    NotYourTurn,
    ReplayFailure,
    UnreplayableGame,
    history_of,
    replay,
)
from .storage import GameStore, StoredGame, default_db_path

STATIC = Path(__file__).parent / "static"


class RevalidatingStaticFiles(StaticFiles):
    """Static files the browser must check for changes on every load.

    Without this, a browser may keep running an old copy of the JavaScript
    after it changes. The check is cheap: an unchanged file gets a 304.
    """

    def file_response(self, *args, **kwargs):
        response = super().file_response(*args, **kwargs)
        response.headers["Cache-Control"] = "no-cache"
        return response


class NewGame(BaseModel):
    mode: Mode = Mode.HOTSEAT


class PlayAction(BaseModel):
    action: str
    version: int


def create_app(db_path: Path | None = None, rng: random.Random | None = None) -> FastAPI:
    store = GameStore(db_path or default_db_path())
    rng = rng or random.Random()
    app = FastAPI(title="Dus Dus Dus")

    @app.post("/api/games", status_code=201)
    def create_game(body: NewGame) -> dict:
        game = Game.new(secrets.token_urlsafe(6), body.mode)
        store.insert(game)
        return game_payload(game)

    @app.get("/api/games")
    def list_games() -> list[dict]:
        return [summary_payload(stored) for stored in store.list_records()]

    @app.get("/api/games/{game_id}")
    def get_game(game_id: str) -> dict:
        return game_payload(_load(game_id))

    @app.get("/api/games/{game_id}/replay")
    def get_replay(game_id: str) -> dict:
        stored = store.load_record(game_id)
        if stored is None:
            raise HTTPException(404, "No such game")
        return replay_payload(stored)

    @app.post("/api/games/{game_id}/actions")
    def play_action(game_id: str, body: PlayAction) -> dict:
        game = _load(game_id)

        if body.version != game.version:
            raise HTTPException(409, "The game has moved on since you last saw it")

        try:
            game.play(body.action, rng)
        except NotYourTurn as error:
            raise HTTPException(409, str(error))
        except ValueError as error:  # bad notation, or an illegal action
            raise HTTPException(400, str(error))

        if not store.update(game, expected_version=body.version):
            raise HTTPException(409, "The game has moved on since you last saw it")

        return game_payload(game)

    def _load(game_id: str) -> Game:
        try:
            game = store.load(game_id)
        except UnreplayableGame as error:
            raise HTTPException(
                409,
                "This game was recorded under older rules and can't be replayed"
                f" under the current ones. {error}.",
            )
        if game is None:
            raise HTTPException(404, "No such game")
        return game

    # Mounted last so the API routes above take precedence.
    app.mount("/", RevalidatingStaticFiles(directory=STATIC, html=True), name="static")
    return app


def state_payload(state: GameState) -> dict:
    pieces = {}
    for x in range(7):
        for y in range(7):
            position = Position(x, y)
            owner = state.piece_at(position)
            if owner is not None:
                pieces[str(position)] = PLAYER_NAMES[owner]

    return {
        "current_player": PLAYER_NAMES[state.current_player],
        "setup": state.setup,
        "turn": state.turn_count,
        "action": state.action_count,
        "winner": _player(state.winner),
        "ball": None if state.ball is None else str(state.ball),
        "pieces": pieces,
    }


def game_payload(game: Game) -> dict:
    return {
        "id": game.id,
        "mode": game.mode.value,
        "version": game.version,
        "human_players": [PLAYER_NAMES[player] for player in game.human_players],
        "state": state_payload(game.state),
        "valid_actions": [str(action) for action in game.state.valid_actions()],
        "history": game.history(),
    }


def summary_payload(stored: StoredGame) -> dict:
    states, failure = replay(stored.record)
    winner = states[-1].winner

    if failure is not None:
        status = "unreplayable"
    elif winner is not None:
        status = "finished"
    else:
        status = "in_progress"

    return {
        "id": stored.id,
        "mode": stored.mode.value,
        "moves": len(stored.record),
        "status": status,
        "winner": _player(winner),
        "problem": None if failure is None else str(failure),
        "created_at": stored.created_at,
        "updated_at": stored.updated_at,
    }


def replay_payload(stored: StoredGame) -> dict:
    """Every position of a game, for stepping through it.

    Frame 0 is the start; frame n is the position after move n. A game the
    current rules forbid stops at the last legal position, with the failure.
    """
    states, failure = replay(stored.record)

    return {
        "id": stored.id,
        "mode": stored.mode.value,
        "frames": [state_payload(state) for state in states],
        "history": history_of(states, stored.record),
        "failure": _failure_payload(failure),
    }


def _failure_payload(failure: ReplayFailure | None) -> dict | None:
    if failure is None:
        return None
    return {
        "move": failure.move,
        "player": PLAYER_NAMES[failure.player],
        "action": failure.action,
        "reason": failure.reason,
    }


def _player(player) -> str | None:
    return None if player is None else PLAYER_NAMES[player]
