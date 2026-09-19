"""SQLite storage for games.

Each row holds a game's record: the list of actions, from which the current
state is rebuilt by replaying. The record is the single source of truth, so a
stored state can never drift from the moves that produced it.
"""

import os
import sqlite3
from contextlib import contextmanager
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from dus_engine import GameRecord

from .games import PLAYER_NAMES, Game, Mode

SCHEMA = """
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    mode TEXT NOT NULL,
    record TEXT NOT NULL,
    version INTEGER NOT NULL,
    winner TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
"""


@dataclass
class StoredGame:
    """A row of the games table, before its moves are replayed."""

    COLUMNS = "id, mode, record, created_at, updated_at"

    id: str
    mode: Mode
    record: GameRecord
    created_at: str
    updated_at: str

    @classmethod
    def from_row(cls, row) -> "StoredGame":
        id, mode, record, created_at, updated_at = row
        return cls(id, Mode(mode), GameRecord.from_json(record), created_at, updated_at)


def default_db_path() -> Path:
    """Where games are kept unless DUS_DB says otherwise.

    Deliberately outside the project directory: a file-syncing service copying
    a live SQLite database mid-write can corrupt it.
    """
    if path := os.environ.get("DUS_DB"):
        return Path(path)
    return Path.home() / ".local" / "share" / "dus-dus-dus" / "games.sqlite"


class GameStore:
    def __init__(self, path: Path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with self._db() as db:
            db.execute(SCHEMA)

    def insert(self, game: Game) -> None:
        now = _now()
        with self._db() as db:
            db.execute(
                "INSERT INTO games (id, mode, record, version, winner, created_at, updated_at)"
                " VALUES (?, ?, ?, ?, ?, ?, ?)",
                (game.id, game.mode.value, game.record.to_json(), game.version,
                 _winner(game), now, now),
            )

    def update(self, game: Game, expected_version: int) -> bool:
        """Save a game only if nobody else saved it since `expected_version`.

        Returns False, changing nothing, if the stored game has moved on.
        """
        with self._db() as db:
            cursor = db.execute(
                "UPDATE games SET record = ?, version = ?, winner = ?, updated_at = ?"
                " WHERE id = ? AND version = ?",
                (game.record.to_json(), game.version, _winner(game), _now(),
                 game.id, expected_version),
            )
            return cursor.rowcount == 1

    def load(self, id: str) -> Game | None:
        """The game with this id, ready to play, or None if there isn't one.

        Raises UnreplayableGame if the current rules forbid one of its moves.
        """
        stored = self.load_record(id)
        if stored is None:
            return None
        return Game.from_record(stored.id, stored.mode, stored.record)

    def load_record(self, id: str) -> StoredGame | None:
        """The game's stored moves, without replaying them."""
        with self._db() as db:
            row = db.execute(
                f"SELECT {StoredGame.COLUMNS} FROM games WHERE id = ?", (id,)
            ).fetchone()

        return None if row is None else StoredGame.from_row(row)

    def list_records(self, limit: int = 50) -> list[StoredGame]:
        """The most recently played games first."""
        with self._db() as db:
            rows = db.execute(
                f"SELECT {StoredGame.COLUMNS} FROM games ORDER BY updated_at DESC LIMIT ?",
                (limit,),
            ).fetchall()

        return [StoredGame.from_row(row) for row in rows]

    @contextmanager
    def _db(self):
        db = sqlite3.connect(self.path)
        try:
            with db:  # commits on success, rolls back on error
                yield db
        finally:
            db.close()


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _winner(game: Game) -> str | None:
    winner = game.state.winner
    return None if winner is None else PLAYER_NAMES[winner]
