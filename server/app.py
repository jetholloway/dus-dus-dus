"""The web server: a JSON API over the engine, plus the static browser client.

Run it with `just serve`, which starts uvicorn on this module's create_app.
"""

import random
import secrets
from pathlib import Path

from dus_engine import Position
from fastapi import FastAPI, HTTPException
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from .games import PLAYER_NAMES, Game, Mode, NotYourTurn
from .storage import GameStore, default_db_path

STATIC = Path(__file__).parent / "static"


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
        return store.summaries()

    @app.get("/api/games/{game_id}")
    def get_game(game_id: str) -> dict:
        return game_payload(_load(game_id))

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
        game = store.load(game_id)
        if game is None:
            raise HTTPException(404, "No such game")
        return game

    # Mounted last so the API routes above take precedence.
    app.mount("/", StaticFiles(directory=STATIC, html=True), name="static")
    return app


def game_payload(game: Game) -> dict:
    state = game.state
    pieces = {}
    for x in range(7):
        for y in range(7):
            position = Position(x, y)
            owner = state.piece_at(position)
            if owner is not None:
                pieces[str(position)] = PLAYER_NAMES[owner]

    return {
        "id": game.id,
        "mode": game.mode.value,
        "version": game.version,
        "human_players": [PLAYER_NAMES[player] for player in game.human_players],
        "state": {
            "current_player": PLAYER_NAMES[state.current_player],
            "setup": state.setup,
            "turn": state.turn_count,
            "action": state.action_count,
            "winner": None if state.winner is None else PLAYER_NAMES[state.winner],
            "ball": None if state.ball is None else str(state.ball),
            "pieces": pieces,
        },
        "valid_actions": [str(action) for action in state.valid_actions()],
        "history": game.history(),
    }
