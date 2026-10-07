"""Write the openings of saved games to a file the arena reads with --openings.

Each opening is a game's first N turns, each side's setup move counting as
a turn, so `--turns 4` matches the arena's `--opening random:4`. Games too
short to have them are left out, as are repeats. The database is only read.

    python experiments/export_openings.py [--turns N] [--db PATH] [--out PATH]
"""

import argparse
import json
import os
import sqlite3
from datetime import date
from pathlib import Path

HERE = Path(__file__).parent


def default_db() -> Path:
    """Where the web server keeps games; see server/storage.py."""
    if path := os.environ.get("DUS_DB"):
        return Path(path)
    return Path.home() / ".local" / "share" / "dus-dus-dus" / "games.sqlite"


def opening_length(turns: int) -> int:
    """Actions in the first `turns` turns: one setup move each, then three."""
    return min(turns, 2) + 3 * max(turns - 2, 0)


def openings(db: Path, turns: int) -> list[list[str]]:
    connection = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    try:
        rows = connection.execute("SELECT record FROM games ORDER BY created_at").fetchall()
    finally:
        connection.close()

    length = opening_length(turns)
    seen = set()
    result = []
    for (record,) in rows:
        actions = json.loads(record)["actions"]
        # Longer than the opening, so the game wasn't already over by then.
        if len(actions) <= length:
            continue
        opening = tuple(actions[:length])
        if opening not in seen:
            seen.add(opening)
            result.append(list(opening))
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--turns", type=int, default=4, help="turns per opening (default 4)")
    parser.add_argument("--db", type=Path, default=default_db(), help="games database")
    parser.add_argument(
        "--out", type=Path, default=HERE / "openings.txt", help="file to write"
    )
    args = parser.parse_args()

    found = openings(args.db, args.turns)
    header = f"# {len(found)} openings of {args.turns} turns from {args.db}, {date.today()}\n"
    args.out.write_text(header + "".join(", ".join(opening) + "\n" for opening in found))
    print(f"wrote {len(found)} openings to {args.out}")


if __name__ == "__main__":
    main()
