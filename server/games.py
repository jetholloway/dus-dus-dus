"""A game in progress: the engine state plus who plays each side."""

import random
from dataclasses import dataclass
from enum import Enum

from dus_engine import Action, GameRecord, GameState, InvalidAction, Player

PLAYER_NAMES = {Player.First: "First", Player.Second: "Second"}

# In a bot game the human plays First and the bot plays Second.
BOT_PLAYER = Player.Second


class Mode(str, Enum):
    HOTSEAT = "hotseat"
    BOT = "bot"


class NotYourTurn(Exception):
    pass


class UnreplayableGame(Exception):
    """A stored game whose moves the current rules no longer allow.

    Games are rebuilt by replaying their moves, so a rule change can leave an
    old game with a move that is now illegal.
    """


@dataclass
class ReplayFailure:
    """The first move of a record that the current rules forbid."""

    move: int  # counted from 1
    player: Player
    action: str
    reason: str

    def __str__(self) -> str:
        return f"Move {self.move} ({self.action}) is not allowed: {self.reason}"


def replay(record: GameRecord) -> tuple[list[GameState], ReplayFailure | None]:
    """The state before the first move and after each one.

    Unlike GameRecord.replay, this stops at the first illegal move rather than
    failing outright, so a game recorded under older rules can still be shown
    up to the point where the rules changed.
    """
    states = [GameState()]

    for index, action in enumerate(record.actions):
        try:
            states.append(states[-1].apply(action))
        except InvalidAction as error:
            failure = ReplayFailure(index + 1, states[-1].current_player, str(action), str(error))
            return states, failure

    return states, None


def history_of(states: list[GameState], record: GameRecord) -> list[dict]:
    """Each replayed move with the player who made it.

    `states` comes from replay(): every state but the last is one a move was
    played from, so a move the replay stopped at is left out.
    """
    return [
        {"player": PLAYER_NAMES[state.current_player], "action": str(action)}
        for state, action in zip(states[:-1], record.actions)
    ]


@dataclass
class Game:
    id: str
    mode: Mode
    record: GameRecord
    state: GameState

    @classmethod
    def new(cls, id: str, mode: Mode) -> "Game":
        return cls(id=id, mode=mode, record=GameRecord(), state=GameState())

    @classmethod
    def from_record(cls, id: str, mode: Mode, record: GameRecord) -> "Game":
        """Rebuild a game by replaying its record, which also validates it."""
        states, failure = replay(record)
        if failure is not None:
            raise UnreplayableGame(str(failure))
        return cls(id=id, mode=mode, record=record, state=states[-1])

    @property
    def version(self) -> int:
        return len(self.record)

    @property
    def human_players(self) -> list[Player]:
        if self.mode is Mode.BOT:
            return [player for player in PLAYER_NAMES if player != BOT_PLAYER]
        return list(PLAYER_NAMES)

    def is_bot_turn(self) -> bool:
        return (
            self.mode is Mode.BOT
            and not self.state.is_terminal
            and self.state.current_player == BOT_PLAYER
        )

    def play(self, notation: str, rng: random.Random) -> None:
        """Apply a human's action, then let the bot reply if it's the bot's turn.

        Raises ValueError for notation that doesn't parse, InvalidAction (a
        ValueError) for an action the rules forbid, and NotYourTurn if the bot
        is due to move.
        """
        if self.is_bot_turn():
            raise NotYourTurn("It's the bot's turn")

        self._apply(Action(notation))

        while self.is_bot_turn():
            actions = self.state.valid_actions()
            if not actions:
                raise RuntimeError("The bot has no legal action in a game that isn't won")
            self._apply(rng.choice(actions))

    def history(self) -> list[dict]:
        """Every action so far, with the player who made it."""
        states, _ = replay(self.record)
        return history_of(states, self.record)

    def _apply(self, action: Action) -> None:
        self.state = self.state.apply(action)
        self.record.push(action)
        self.record.winner = self.state.winner
