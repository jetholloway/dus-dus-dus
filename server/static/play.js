// The play view: a game in progress.
//
// The server owns the game and sends its state along with every legal action.
// This view only draws the board and turns clicks into one of those actions,
// so it never needs to know the rules.

import { MODES, NAMES, api, playerName, setMessage } from "./api.js";
import { actionSquares, drawBoard } from "./board.js";

let game = null;
// The square of the piece the player has picked, or null.
let selected = null;
// True once the player has clicked the ball to tackle it with `selected`.
let tackling = false;
// Shown once the next game loads, such as which side the player is on.
let greeting = "";

const element = (id) => document.getElementById(id);
const message = (text, isError) => setMessage(element("play-message"), text, isError);

export async function startGame(mode) {
  try {
    const created = await api("/games", {
      method: "POST",
      body: JSON.stringify({ mode, name: playerName() }),
    });
    greeting = mode === "bot" ? "You are Orange. The bot plays Teal." : "";
    location.hash = `play/${created.id}`;
  } catch (error) {
    message(error.message, true);
  }
}

// Accepts an invitation, then opens the game to play.
export async function joinGame(id, invite) {
  try {
    const joined = await api(`/games/${id}/join`, {
      method: "POST",
      body: JSON.stringify({ invite, name: playerName() }),
    });
    const side = Object.keys(joined.seats).find((seat) => joined.seats[seat].is_you);
    greeting = side ? `You joined as ${NAMES[side]}.` : "";
  } catch (error) {
    greeting = error.message;
  }

  location.hash = `play/${id}`;
}

export async function showPlay(id) {
  game = null;
  clearSelection();
  message(greeting);
  greeting = "";

  const replayLink = element("play-replay");
  replayLink.href = `#replay/${id}`;
  element("play-invite").hidden = true;

  try {
    game = await api(`/games/${id}`);
  } catch (error) {
    if (error.status === 404) {
      message("That game doesn't exist.", true);
    } else {
      // A game the current rules can't replay can still be watched up to that point.
      message(error.message, true);
    }
  }

  render();
}

export function playKey(event) {
  if (event.key === "Escape") {
    clearSelection();
    render();
  }
}

function parseActions() {
  if (!canAct()) {
    return [];
  }

  return game.valid_actions.map((notation) => {
    const [type, src, dst] = notation.split(" ");
    return { type, src, dst, notation };
  });
}

function canAct() {
  return (
    game !== null &&
    game.state.winner === null &&
    game.can_play.includes(game.state.current_player)
  );
}

function clearSelection() {
  selected = null;
  tackling = false;
}

// Clicks

function onSquare(square) {
  const actions = parseActions();

  if (selected === null) {
    if (actions.some((action) => action.src === square)) {
      selected = square;
    }
    render();
    return;
  }

  if (square === selected) {
    clearSelection();
    render();
    return;
  }

  const fromSelected = actions.filter((action) => action.src === selected);
  const find = (type) =>
    fromSelected.find((action) => action.type === type && action.dst === square);

  if (tackling) {
    const tackle = find("TACKLE");
    if (tackle) {
      submit(tackle.notation);
      return;
    }
  } else if (
    square === game.state.ball &&
    fromSelected.some((action) => action.type === "TACKLE")
  ) {
    tackling = true;
    render();
    return;
  } else {
    const action = find("MOVE") ?? find("PASS");
    if (action) {
      submit(action.notation);
      return;
    }
  }

  // Anything else re-selects: another of the player's pieces, or nothing.
  tackling = false;
  selected = actions.some((action) => action.src === square) ? square : null;
  render();
}

async function submit(notation) {
  const before = game.history.length;

  try {
    game = await api(`/games/${game.id}/actions`, {
      method: "POST",
      body: JSON.stringify({ action: notation, version: game.version }),
    });
    message(opponentSummary(before));
  } catch (error) {
    message(error.message, true);
    if (error.status === 409) {
      game = await api(`/games/${game.id}`);
    }
  }

  clearSelection();
  render();
}

// Describes what the bot did in reply, if anything.
function opponentSummary(before) {
  const replies = game.history
    .slice(before)
    .filter((entry) => game.seats[entry.player].is_bot);

  if (replies.length === 0) {
    return "";
  }

  const player = NAMES[replies[0].player];
  return `${player} (bot) played ${replies.map((entry) => entry.action).join(", ")}.`;
}

// Drawing

function render() {
  renderStatus();
  renderInvite();
  renderBoard();
  renderLog();
}

// The link that gives the empty seat of an online game to a friend.
function renderInvite() {
  const invite = element("play-invite");
  invite.hidden = game === null || !game.invite;

  if (!invite.hidden) {
    const { origin, pathname } = location;
    element("play-invite-link").value = `${origin}${pathname}#join/${game.id}/${game.invite}`;
  }
}

export function setUpInvite() {
  const link = element("play-invite-link");

  element("play-invite-copy").addEventListener("click", async () => {
    link.select();
    try {
      await navigator.clipboard.writeText(link.value);
      message("Invite link copied.");
    } catch {
      // Copying needs a secure page, which a plain http:// address is not.
      message("Press Ctrl+C to copy the selected link.");
    }
  });
}

function renderStatus() {
  const status = element("play-status");

  if (game === null) {
    status.textContent = "";
    element("play-mode").textContent = "";
    return;
  }

  const { state } = game;
  const player = describeSeat(state.current_player);

  const waitingFor = Object.keys(game.seats).find((seat) => game.seats[seat].name === null);

  if (state.winner !== null) {
    status.textContent = `${describeSeat(state.winner)} wins!`;
  } else if (game.invite && waitingFor) {
    status.textContent = `Waiting for someone to join as ${NAMES[waitingFor]}.`;
  } else if (state.setup) {
    const ball = state.current_player === "First" ? " It will hold the ball." : "";
    status.textContent =
      `Setup: ${player}, choose a champion and move it two squares forward.${ball}`;
  } else if (tackling) {
    status.textContent = `${player}: choose where the tackler goes.`;
  } else {
    status.textContent =
      `${player} to play: action ${state.action} of 3, turn ${state.turn}.`;
  }

  element("play-mode").textContent = `${MODES[game.mode]} — ${describeSeat("First")} vs ${describeSeat("Second")}`;
}

// "Orange (you)", "Teal (Bot)", or just "Orange" for a seat nobody has taken.
function describeSeat(side) {
  const seat = game.seats[side];
  const who = seat.is_you ? "you" : seat.name;
  return who ? `${NAMES[side]} (${who})` : NAMES[side];
}

function renderBoard() {
  const board = element("play-board");

  if (game === null) {
    board.replaceChildren();
    return;
  }

  const actions = parseActions();
  const lastEntry = game.history.at(-1);

  drawBoard(board, game.state, {
    onSquare,
    selected,
    targets: targetSquares(actions),
    movable: selected === null ? new Set(actions.map((action) => action.src)) : new Set(),
    last: lastEntry ? actionSquares(lastEntry.action) : new Set(),
  });
}

// The squares to highlight for the selected piece, mapped to a CSS class.
function targetSquares(actions) {
  const targets = new Map();

  if (selected === null) {
    return targets;
  }

  const fromSelected = actions.filter((action) => action.src === selected);

  if (tackling) {
    for (const action of fromSelected) {
      if (action.type === "TACKLE") {
        targets.set(action.dst, "tackle-dst");
      }
    }
    targets.set(game.state.ball, "tackle-target");
    return targets;
  }

  for (const action of fromSelected) {
    if (action.type === "MOVE") {
      targets.set(action.dst, "move-target");
    } else if (action.type === "PASS") {
      targets.set(action.dst, "pass-target");
    }
  }

  if (fromSelected.some((action) => action.type === "TACKLE")) {
    targets.set(game.state.ball, "tackle-target");
  }

  return targets;
}

function renderLog() {
  const log = element("play-log");
  log.replaceChildren();

  if (game === null) {
    return;
  }

  for (const entry of game.history) {
    const item = document.createElement("li");
    item.className = entry.player.toLowerCase();
    item.textContent = `${NAMES[entry.player]}: ${entry.action}`;
    log.append(item);
  }

  log.scrollTop = log.scrollHeight;
}
