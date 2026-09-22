"""A game in progress: the engine state plus who plays each side."""

import random
import secrets
from dataclasses import dataclass, field
from enum import Enum

from dus_engine import Action, GameRecord, GameState, InvalidAction, Player

PLAYER_NAMES = {Player.First: "First", Player.Second: "Second"}
PLAYERS = {name: player for player, name in PLAYER_NAMES.items()}

# The owner recorded for a seat the bot plays.
BOT = "bot"
BOT_NAME = "Bot"


class Mode(str, Enum):
    HOTSEAT = "hotseat"
    BOT = "bot"
    ONLINE = "online"


class NotYourTurn(Exception):
    pass


class BadInvite(Exception):
    pass


class SeatTaken(Exception):
    pass


class UnreplayableGame(Exception):
    """A stored game whose moves the current rules no longer allow.

    Games are rebuilt by replaying their moves, so a rule change can leave an
    old game with a move that is now illegal.
    """


@dataclass
class Seat:
    """Who plays one side of a game.

    An owner of None means the seat is open: anyone may play it. Games from
    before seats existed have two open seats, which keeps them playable.
    """

    owner: str | None = None  # a browser's player id, or BOT
    name: str | None = None

    @property
    def is_bot(self) -> bool:
        return self.owner == BOT

    def belongs_to(self, player_id: str | None) -> bool:
        return self.owner is not None and self.owner == player_id

    def playable_by(self, player_id: str | None) -> bool:
        return self.owner is None or self.belongs_to(player_id)


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
    seats: dict[Player, Seat] = field(default_factory=dict)
    # The code that claims the empty seat of an online game, until someone does.
    invite: str | None = None

    @classmethod
    def new(
        cls,
        id: str,
        mode: Mode,
        player_id: str,
        player_name: str,
        rng: random.Random | None = None,
    ) -> "Game":
        """A new game, with its seats given to whoever asked for it.

        In hot-seat both seats are theirs, since one browser plays both sides.
        Against the bot they take Orange and the bot takes Teal. Online they
        take one side at random and the other waits for a friend to join.
        """
        player = Seat(player_id, player_name)
        seats = {Player.First: player, Player.Second: player}
        invite = None

        if mode is Mode.BOT:
            seats[Player.Second] = Seat(BOT, BOT_NAME)
        elif mode is Mode.ONLINE:
            theirs = (rng or random.Random()).choice(list(PLAYER_NAMES))
            seats[theirs] = Seat()
            invite = secrets.token_urlsafe(8)

        return cls(
            id=id,
            mode=mode,
            record=GameRecord(),
            state=GameState(),
            seats=seats,
            invite=invite,
        )

    @classmethod
    def from_record(
        cls,
        id: str,
        mode: Mode,
        record: GameRecord,
        seats: dict[Player, Seat] | None = None,
        invite: str | None = None,
    ) -> "Game":
        """Rebuild a game by replaying its record, which also validates it."""
        states, failure = replay(record)
        if failure is not None:
            raise UnreplayableGame(str(failure))
        return cls(
            id=id,
            mode=mode,
            record=record,
            state=states[-1],
            seats=seats or {},
            invite=invite,
        )

    @property
    def version(self) -> int:
        return len(self.record)

    def seat(self, player: Player) -> Seat:
        return self.seats.get(player, Seat())

    def playable_seats(self, player_id: str | None) -> list[Player]:
        """The sides this player may move, which is what the UI acts on.

        A seat still waiting for an invitation to be accepted is nobody's to
        play, unlike the open seats of a game saved before seats existed.
        """
        return [
            player
            for player in PLAYER_NAMES
            if not self.seat(player).is_bot
            and self.seat(player).playable_by(player_id)
            and not (self.invite is not None and self.seat(player).owner is None)
        ]

    def open_seat(self) -> Player | None:
        """The side waiting for someone, if any."""
        for player in PLAYER_NAMES:
            if self.seat(player).owner is None:
                return player
        return None

    def join(self, player_id: str, player_name: str, invite: str) -> None:
        """Take the open seat of an online game.

        Opening a used invite again is harmless for whoever already holds a
        seat, so a friend can reload the link they were sent.
        """
        if any(seat.belongs_to(player_id) for seat in self.seats.values()):
            return

        if self.invite is None or not secrets.compare_digest(invite, self.invite):
            raise BadInvite("That invitation is not valid for this game")

        seat = self.open_seat()
        if seat is None:
            raise SeatTaken("Both sides of this game are taken")

        self.seats[seat] = Seat(player_id, player_name)
        self.invite = None

    def is_bot_turn(self) -> bool:
        return not self.state.is_terminal and self.seat(self.state.current_player).is_bot

    def play(self, notation: str, rng: random.Random, player_id: str | None = None) -> None:
        """Apply a player's action, then let the bot reply if it's its turn.

        Raises ValueError for notation that doesn't parse, InvalidAction (a
        ValueError) for an action the rules forbid, and NotYourTurn if the
        move isn't this player's to make.
        """
        if self.is_bot_turn():
            raise NotYourTurn("It's the bot's turn")

        if not self.state.is_terminal:
            seat = self.seat(self.state.current_player)
            if not seat.playable_by(player_id):
                raise NotYourTurn(f"It's {seat.name or 'the other player'}'s turn")

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
