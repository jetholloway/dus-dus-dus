"""Run a named battery of arena matches and save the results.

    python experiments/run.py                     # list the experiments
    python experiments/run.py NAME [--trials N] [--seed S] [--rules RULES]
                                   [--opening SPEC | --openings FILE]

Each experiment is a function below, registered with @experiment, which
also sets the opening it starts from unless --opening or --openings says
otherwise. It plays matches through `match()`, which runs the release arena
binary with --json, and returns a list of result rows. Every row is saved, flattened, to
experiments/results/NAME-<time>.csv, and the experiment prints its own
summary as it goes.
"""

import argparse
import csv
import json
import math
import subprocess
import sys
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).parent
ROOT = HERE.parent
BINARY = ROOT / "target" / "release" / "dus_dus_dus"
RESULTS = HERE / "results"

EXPERIMENTS = {}


def experiment(opening: str):
    """Register an experiment under its name, with underscores as hyphens,
    starting from `opening` (an --opening value) by default."""

    def register(function):
        function.opening = opening
        EXPERIMENTS[function.__name__.replace("_", "-")] = function
        return function

    return register


def wilson(successes: int, trials: int) -> tuple[float, float]:
    """A 95% confidence interval for a proportion, as the arena reports."""
    if trials == 0:
        return 0.0, 1.0
    z, n = 1.96, trials
    p = successes / n
    denominator = 1 + z * z / n
    centre = (p + z * z / (2 * n)) / denominator
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denominator
    return max(centre - half, 0.0), min(centre + half, 1.0)


class Arena:
    """Runs arena matches with the settings the command line gave."""

    def __init__(self, trials: int, seed: int, opening: list[str], rules: str):
        self.trials = trials
        self.seed = seed
        self.opening = opening
        self.rules = rules

    def match(
        self, a: str, b: str, trials: int | None = None, rules: str | None = None
    ) -> dict:
        """Play bot A against bot B and return the arena's JSON results."""
        command = [
            str(BINARY), "arena", a, b,
            "--trials", str(trials or self.trials),
            "--seed", str(self.seed),
            "--rules", rules or self.rules,
            *self.opening,
            "--json",
        ]  # fmt: skip
        result = subprocess.run(command, capture_output=True, text=True)
        if result.returncode != 0:
            sys.exit(f"arena failed: {' '.join(command)}\n{result.stderr}")
        return json.loads(result.stdout)


@experiment(opening="random:4")
def heuristic_strength_grid(arena: Arena) -> list[dict]:
    """Every heuristic strength (pieces=1 to 7) against every other.

    Prints a grid of win rates: the row's strength against the column's. A
    strength against itself should come out near 50%, which checks that
    trials are fair. Each pair is played once, since B's win rate against A
    is A's loss rate.
    """
    strengths = range(1, 8)
    name = lambda strength: f"heuristic:pieces={strength}"
    pairs = [(i, j) for i in strengths for j in strengths if i <= j]
    rate = {}
    rows = []

    for number, (i, j) in enumerate(pairs, 1):
        result = arena.match(name(i), name(j))
        rate[i, j] = result["a"]["win_rate"]
        rate[j, i] = result["b"]["win_rate"] if i != j else rate[i, j]
        rows.append(result)
        print(
            f"{number:>2}/{len(pairs)}  {i} vs {j}: {100 * rate[i, j]:5.1f}% to"
            f" {100 * result['b']['win_rate']:5.1f}%,"
            f" draws {result['draws']}, splits {result['pairs']['split']}",
            flush=True,
        )

    print("\nWin rate of the row's strength against the column's:\n")
    print("      " + "".join(f"{j:>7}" for j in strengths))
    for i in strengths:
        print(f"{i:>6}" + "".join(f"{100 * rate[i, j]:>6.1f}%" for j in strengths))
    return rows


def orange_wins(arena: Arena, bot: str, rules: str | None = None) -> dict:
    """A bot against itself, with how often Orange won added to the results."""
    result = arena.match(bot, bot, rules=rules)
    orange = result["a"]["wins_as_first"] + result["b"]["wins_as_first"]
    low, high = wilson(orange, result["games"])
    result["orange_wins"] = orange
    result["orange_win_rate"] = orange / result["games"]
    result["orange_win_rate_low"] = low
    result["orange_win_rate_high"] = high
    return result


def percent_range(result: dict) -> str:
    """Orange's win rate and its 95% range, e.g. "64.3% (61.3–67.2)"."""
    return (
        f"{100 * result['orange_win_rate']:.1f}%"
        f" ({100 * result['orange_win_rate_low']:.1f}–{100 * result['orange_win_rate_high']:.1f})"
    )


@experiment(opening="start")
def first_player_advantage(arena: Arena) -> list[dict]:
    """How often Orange, who moves first, wins when both sides play alike.

    Each bot plays itself, so any difference between the colours comes from
    the colours alone. From the normal start by default: a random opening
    always ends with Orange to move, which would mix in whatever position
    random play left behind.
    """
    bots = ["random"] + [f"heuristic:pieces={strength}" for strength in range(1, 8)]
    rows = []

    print(f"{'bot':<20} {'Orange wins (95% range)':>24} {'draws':>6} {'length':>7}")
    for bot in bots:
        result = orange_wins(arena, bot)
        rows.append(result)
        print(
            f"{bot:<20} {percent_range(result):>24}"
            f" {result['draws']:>6} {result['mean_game_length']:>7.0f}",
            flush=True,
        )
    return rows


@experiment(opening="start")
def rule_variants(arena: Arena) -> list[dict]:
    """Orange's win rate under each setup rule variant, for a few bots.

    Like first-player-advantage, each bot plays itself. A fair variant
    brings Orange's win rate to about 50% for the bots that play well. The
    --rules option is ignored: every variant is played.
    """
    variants = [
        "standard",
        "teal-ball",
        "teal-double-setup",
        "teal-ball+teal-double-setup",
        "teal-triple-setup",
        "teal-ball+teal-triple-setup",
        "teal-ball+orange-double-setup",
    ]
    bots = ["random", "heuristic:pieces=4", "heuristic:pieces=7"]
    rows = []

    for rules in variants:
        print(rules)
        for bot in bots:
            result = orange_wins(arena, bot, rules)
            rows.append(result)
            print(f"    {bot:<20} Orange wins {percent_range(result)}", flush=True)

    print("\nOrange's win rate:\n")
    print(f"{'rules':<30}" + "".join(f"{bot:>22}" for bot in bots))
    for rules in variants:
        cells = [row for row in rows if row["rules"] == rules]
        print(f"{rules:<30}" + "".join(f"{percent_range(row):>22}" for row in cells))
    return rows


def flatten(row: dict, prefix: str = "") -> dict:
    """{"a": {"wins": 3}} becomes {"a_wins": 3}, for a CSV."""
    flat = {}
    for key, value in row.items():
        if isinstance(value, dict):
            flat.update(flatten(value, f"{prefix}{key}_"))
        else:
            flat[f"{prefix}{key}"] = value
    return flat


def save(name: str, rows: list[dict]) -> Path:
    RESULTS.mkdir(exist_ok=True)
    path = RESULTS / f"{name}-{datetime.now():%Y%m%d-%H%M%S}.csv"
    flat = [flatten(row) for row in rows]
    with path.open("w", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=list(flat[0]))
        writer.writeheader()
        writer.writerows(flat)
    return path


def build() -> None:
    """Build the release arena, so every match runs the current code."""
    command = ["cargo", "build", "--release", "-q", "-p", "dus-dus-dus"]
    if subprocess.run(command, cwd=ROOT).returncode != 0:
        sys.exit("cargo build failed")


def main() -> None:
    parser = argparse.ArgumentParser(description="Run a battery of arena matches.")
    parser.add_argument("name", nargs="?", help="experiment to run; omit to list them")
    parser.add_argument("--trials", type=int, default=500, help="trials per match (default 500)")
    parser.add_argument("--seed", type=int, default=0, help="arena seed (default 0)")
    parser.add_argument(
        "--rules", default="standard", help="game rules for every match (default standard)"
    )
    openings = parser.add_mutually_exclusive_group()
    openings.add_argument(
        "--opening", help="start or random:N (default: the experiment's own)"
    )
    openings.add_argument("--openings", type=Path, help="file of openings, one per line")
    args = parser.parse_args()

    if args.name is None:
        for name, function in EXPERIMENTS.items():
            print(f"{name} (opening {function.opening})")
            print(f"    {function.__doc__.splitlines()[0]}")
        return
    if args.name not in EXPERIMENTS:
        sys.exit(f"unknown experiment {args.name!r}; run with no name to list them")

    function = EXPERIMENTS[args.name]
    opening = (
        ["--openings", str(args.openings.resolve())]
        if args.openings
        else ["--opening", args.opening or function.opening]
    )
    build()
    started = datetime.now()
    print(
        f"{args.name}: {args.trials} trials per match, {' '.join(opening)},"
        f" rules {args.rules}\n"
    )
    rows = function(Arena(args.trials, args.seed, opening, args.rules))
    print(f"\nsaved {save(args.name, rows)} after {datetime.now() - started}")


if __name__ == "__main__":
    main()
