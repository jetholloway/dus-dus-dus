"""SQLite storage for games.

Each row holds a game's record: the list of actions, from which the current
state is rebuilt by replaying. The record is the single source of truth, so a
stored state can never drift from the moves that produced it. Alongside it sits
who holds each seat.
"""

import os
import sqlite3
from contextlib import contextmanager
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

from dus_engine import GameRecord, Player

from .games import PLAYER_NAMES, Game, Mode, Seat

# Bumped whenever the tables change; _migrate brings older files up to it.
SCHEMA_VERSION = 2

GAMES_TABLE = """
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

# Added in schema version 1. Games saved before it have no seats, which leaves
# both sides open for anyone to play. "invite", added in version 2, holds the
# code that claims an online game's empty seat until someone does.
SEAT_COLUMNS = ("first_player", "first_name", "second_player", "second_name", "invite")


@dataclass
class StoredGame:
    """A row of the games table, before its moves are replayed."""

    COLUMNS = "id, mode, record, created_at, updated_at, " + ", ".join(SEAT_COLUMNS)

    id: str
    mode: Mode
    record: GameRecord
    created_at: str
    updated_at: str
    seats: dict[Player, Seat] = field(default_factory=dict)
    invite: str | None = None

    @classmethod
    def from_row(cls, row) -> "StoredGame":
        (id, mode, record, created_at, updated_at,
         first, first_name, second, second_name, invite) = row
        seats = {
            Player.First: Seat(first, first_name),
            Player.Second: Seat(second, second_name),
        }
        return cls(
            id, Mode(mode), GameRecord.from_json(record), created_at, updated_at, seats, invite
        )


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
            _migrate(db)

    def insert(self, game: Game) -> None:
        now = _now()
        with self._db() as db:
            db.execute(
                "INSERT INTO games (id, mode, record, version, winner, created_at, updated_at,"
                f" {', '.join(SEAT_COLUMNS)})"
                " VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (game.id, game.mode.value, game.record.to_json(), game.version,
                 _winner(game), now, now, *_seat_values(game)),
            )

    def update(self, game: Game, expected_version: int) -> bool:
        """Save a game only if nobody else saved it since `expected_version`.

        Returns False, changing nothing, if the stored game has moved on.
        """
        with self._db() as db:
            cursor = db.execute(
                "UPDATE games SET record = ?, version = ?, winner = ?, updated_at = ?,"
                f" {', '.join(f'{column} = ?' for column in SEAT_COLUMNS)}"
                " WHERE id = ? AND version = ?",
                (game.record.to_json(), game.version, _winner(game), _now(),
                 *_seat_values(game), game.id, expected_version),
            )
            return cursor.rowcount == 1

    def load(self, id: str) -> Game | None:
        """The game with this id, ready to play, or None if there isn't one.

        Raises UnreplayableGame if the current rules forbid one of its moves.
        """
        stored = self.load_record(id)
        if stored is None:
            return None
        return Game.from_record(
            stored.id, stored.mode, stored.record, stored.seats, stored.invite
        )

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


def _migrate(db) -> None:
    """Bring a database, new or existing, up to the current schema."""
    db.execute(GAMES_TABLE)

    if db.execute("PRAGMA user_version").fetchone()[0] >= SCHEMA_VERSION:
        return

    columns = {row[1] for row in db.execute("PRAGMA table_info(games)")}
    for column in SEAT_COLUMNS:
        if column not in columns:
            db.execute(f"ALTER TABLE games ADD COLUMN {column} TEXT")

    db.execute(f"PRAGMA user_version = {SCHEMA_VERSION}")


def _seat_values(game: Game) -> tuple:
    return (
        game.seat(Player.First).owner,
        game.seat(Player.First).name,
        game.seat(Player.Second).owner,
        game.seat(Player.Second).name,
        game.invite,
    )


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _winner(game: Game) -> str | None:
    winner = game.state.winner
    return None if winner is None else PLAYER_NAMES[winner]
