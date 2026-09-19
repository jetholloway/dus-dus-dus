// Browser client for the Dus Dus Dus server.
//
// The server owns the game and sends its state along with every legal action.
// This file only draws the board and turns clicks into one of those actions,
// so it never needs to know the rules.

const FILES = "ABCDEFG";
const NAMES = { First: "Orange", Second: "Teal" };

let game = null;
// The square of the piece the player has picked, or null.
let selected = null;
// True once the player has clicked the ball to tackle it with `selected`.
let tackling = false;

const element = (id) => document.getElementById(id);

async function api(path, options = {}) {
  const response = await fetch(`/api${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });
  const body = await response.json();

  if (!response.ok) {
    const error = new Error(body.detail ?? response.statusText);
    error.status = response.status;
    throw error;
  }

  return body;
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
    game.human_players.includes(game.state.current_player)
  );
}

function clearSelection() {
  selected = null;
  tackling = false;
}

function setMessage(text, isError = false) {
  const message = element("message");
  message.textContent = text;
  message.classList.toggle("error", isError);
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
    setMessage(opponentSummary(before));
  } catch (error) {
    setMessage(error.message, true);
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
    .filter((entry) => !game.human_players.includes(entry.player));

  if (replies.length === 0) {
    return "";
  }

  const player = NAMES[replies[0].player];
  return `${player} (bot) played ${replies.map((entry) => entry.action).join(", ")}.`;
}

async function newGame(mode) {
  try {
    game = await api("/games", {
      method: "POST",
      body: JSON.stringify({ mode }),
    });
    history.replaceState(null, "", `#${game.id}`);
    setMessage(mode === "bot" ? "You are Orange. The bot plays Teal." : "");
  } catch (error) {
    setMessage(error.message, true);
  }

  clearSelection();
  render();
}

// Drawing

function render() {
  renderStatus();
  renderBoard();
  renderLog();
}

function renderStatus() {
  const status = element("status");

  if (game === null) {
    status.textContent = "Start a game to play.";
    return;
  }

  const { state } = game;
  const player = NAMES[state.current_player];
  const you =
    game.mode === "bot" && game.human_players.includes(state.current_player)
      ? " (you)"
      : "";

  if (state.winner !== null) {
    status.textContent = `${NAMES[state.winner]} wins!`;
  } else if (state.setup) {
    const ball = state.current_player === "First" ? " It will hold the ball." : "";
    status.textContent =
      `Setup: ${player}${you}, choose a champion and move it two squares forward.${ball}`;
  } else if (tackling) {
    status.textContent = `${player}${you}: choose where the tackler goes.`;
  } else {
    status.textContent =
      `${player}${you} to play: action ${state.action} of 3, turn ${state.turn}.`;
  }
}

function renderBoard() {
  const board = element("board");
  board.replaceChildren();

  if (game === null) {
    return;
  }

  const { state } = game;
  const actions = parseActions();
  const targets = targetSquares(actions);
  const movable = new Set(actions.map((action) => action.src));
  const last = lastActionSquares();

  for (let rank = 7; rank >= 1; rank--) {
    board.append(label(rank, "rank"));

    for (const file of FILES) {
      const square = `${file}${rank}`;
      const owner = state.pieces[square];
      const cell = document.createElement("button");
      cell.type = "button";
      cell.className = "square";

      if (owner) {
        const piece = document.createElement("span");
        piece.className = `piece ${owner.toLowerCase()}`;
        piece.classList.toggle("has-ball", state.ball === square);
        cell.append(piece);
      }

      cell.classList.toggle("selected", square === selected);
      cell.classList.toggle("movable", selected === null && movable.has(square));
      cell.classList.toggle("last", last.has(square));
      if (targets.has(square)) {
        cell.classList.add(targets.get(square));
      }

      cell.setAttribute("aria-label", describe(square, owner));
      cell.addEventListener("click", () => onSquare(square));
      board.append(cell);
    }
  }

  board.append(label("", "file"));
  for (const file of FILES) {
    board.append(label(file, "file"));
  }
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

function lastActionSquares() {
  const last = game.history.at(-1);
  if (!last) {
    return new Set();
  }
  const [, src, dst] = last.action.split(" ");
  return new Set([src, dst]);
}

function describe(square, owner) {
  if (!owner) {
    return `${square}, empty`;
  }
  const ball = game.state.ball === square ? ", holding the ball" : "";
  return `${square}, ${NAMES[owner]} piece${ball}`;
}

function label(text, kind) {
  const span = document.createElement("span");
  span.className = `label ${kind}`;
  span.textContent = text;
  return span;
}

function renderLog() {
  const log = element("log");
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

// Start up

async function start() {
  element("new-bot").addEventListener("click", () => newGame("bot"));
  element("new-hotseat").addEventListener("click", () => newGame("hotseat"));

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      clearSelection();
      render();
    }
  });

  const id = location.hash.slice(1);
  if (id) {
    try {
      game = await api(`/games/${id}`);
    } catch (error) {
      setMessage(error.status === 404 ? "That game doesn't exist." : error.message, true);
    }
  }

  render();
}

start();
