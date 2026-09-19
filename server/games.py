"""A game in progress: the engine state plus who plays each side."""

import random
from dataclasses import dataclass
from enum import Enum

from dus_engine import Action, GameRecord, GameState, Player

PLAYER_NAMES = {Player.First: "First", Player.Second: "Second"}

# In a bot game the human plays First and the bot plays Second.
BOT_PLAYER = Player.Second


class Mode(str, Enum):
    HOTSEAT = "hotseat"
    BOT = "bot"


class NotYourTurn(Exception):
    pass


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
        return cls(id=id, mode=mode, record=record, state=record.replay()[-1])

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
        states = self.record.replay()
        return [
            {"player": PLAYER_NAMES[state.current_player], "action": str(action)}
            for state, action in zip(states, self.record.actions)
        ]

    def _apply(self, action: Action) -> None:
        self.state = self.state.apply(action)
        self.record.push(action)
        self.record.winner = self.state.winner
