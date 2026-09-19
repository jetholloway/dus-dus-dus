// The replay view: stepping through a recorded game.
//
// The server sends every position of the game up front, so stepping is
// instant and needs no further requests.

import { MODES, NAMES, api, setMessage } from "./api.js";
import { actionSquares, drawBoard } from "./board.js";

const AUTOPLAY_DELAY_MS = 700;

let replay = null;
// Which position is showing: 0 is the start, n is after move n.
let frame = 0;
let timer = null;

const element = (id) => document.getElementById(id);
const message = (text, isError) => setMessage(element("replay-message"), text, isError);

export async function showReplay(id) {
  stopReplay();
  replay = null;
  frame = 0;
  message("");

  try {
    replay = await api(`/games/${id}/replay`);
    frame = lastFrame();
  } catch (error) {
    message(error.status === 404 ? "That game doesn't exist." : error.message, true);
  }

  render();
}

export function stopReplay() {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
  element("replay-play").textContent = "Play";
}

export function replayKey(event) {
  const keys = {
    ArrowLeft: () => goTo(frame - 1),
    ArrowRight: () => goTo(frame + 1),
    Home: () => goTo(0),
    End: () => goTo(lastFrame()),
  };

  if (replay !== null && keys[event.key]) {
    event.preventDefault();
    stopReplay();
    keys[event.key]();
  }
}

export function setUpReplayControls() {
  element("replay-first").addEventListener("click", () => step(() => 0));
  element("replay-prev").addEventListener("click", () => step(() => frame - 1));
  element("replay-next").addEventListener("click", () => step(() => frame + 1));
  element("replay-last").addEventListener("click", () => step(lastFrame));
  element("replay-play").addEventListener("click", toggleAutoplay);
  element("replay-slider").addEventListener("input", (event) =>
    step(() => Number(event.target.value)),
  );
}

function step(target) {
  stopReplay();
  goTo(target());
}

function goTo(target) {
  if (replay === null) {
    return;
  }
  frame = Math.max(0, Math.min(lastFrame(), target));
  render();
}

function lastFrame() {
  return replay.frames.length - 1;
}

function toggleAutoplay() {
  if (replay === null) {
    return;
  }

  if (timer !== null) {
    stopReplay();
    return;
  }

  if (frame === lastFrame()) {
    frame = 0;
  }

  element("replay-play").textContent = "Pause";
  timer = setInterval(() => {
    if (frame >= lastFrame()) {
      stopReplay();
      return;
    }
    goTo(frame + 1);
  }, AUTOPLAY_DELAY_MS);
}

// Drawing

function render() {
  const board = element("replay-board");
  const controls = element("replay-controls");

  if (replay === null) {
    board.replaceChildren();
    element("replay-title").textContent = "";
    element("replay-status").textContent = "";
    element("replay-log").replaceChildren();
    controls.hidden = true;
    element("replay-continue").hidden = true;
    return;
  }

  controls.hidden = false;
  const state = replay.frames[frame];
  const move = frame > 0 ? replay.history[frame - 1] : null;

  drawBoard(board, state, { last: move ? actionSquares(move.action) : new Set() });

  const slider = element("replay-slider");
  slider.max = lastFrame();
  slider.value = frame;

  renderTitle();
  renderStatus(state, move);
  renderLog();
}

function renderTitle() {
  const final = replay.frames.at(-1);
  let result;

  if (replay.failure !== null) {
    result = "stopped: recorded under older rules";
  } else if (final.winner !== null) {
    result = `${NAMES[final.winner]} won`;
  } else {
    result = "in progress";
  }

  element("replay-title").textContent =
    `Replay, ${MODES[replay.mode]}, ${replay.history.length} moves, ${result}`;

  const canContinue = replay.failure === null && final.winner === null;
  const link = element("replay-continue");
  link.hidden = !canContinue;
  link.href = `#play/${replay.id}`;
}

function renderStatus(state, move) {
  const status = element("replay-status");

  if (move === null) {
    status.textContent = "Start of the game.";
  } else {
    status.textContent =
      `Move ${frame} of ${replay.history.length}: ${NAMES[move.player]} ${move.action}.`;
  }

  if (frame === lastFrame()) {
    if (replay.failure !== null) {
      const { move: number, player, action, reason } = replay.failure;
      message(
        `Move ${number}, ${NAMES[player]} ${action}, is not allowed under the current` +
          ` rules (${reason}), so the replay stops here.`,
        true,
      );
    } else if (state.winner !== null) {
      message(`${NAMES[state.winner]} wins.`);
    } else {
      message("");
    }
  } else {
    message("");
  }
}

function renderLog() {
  const log = element("replay-log");
  log.replaceChildren();

  replay.history.forEach((entry, index) => {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `${NAMES[entry.player]}: ${entry.action}`;
    button.addEventListener("click", () => step(() => index + 1));
    item.className = entry.player.toLowerCase();
    item.classList.toggle("current", index + 1 === frame);
    item.append(button);
    log.append(item);
  });

  if (replay.failure !== null) {
    const item = document.createElement("li");
    item.className = "failed";
    item.textContent =
      `${NAMES[replay.failure.player]}: ${replay.failure.action} (not allowed)`;
    log.append(item);
  }

  keepInView(log, log.querySelector(".current"));
}

// Scrolls the move list, and only the move list, to show `item`.
// (scrollIntoView would scroll the whole page too.)
function keepInView(list, item) {
  if (item === null) {
    return;
  }

  const top = item.offsetTop;
  const bottom = top + item.offsetHeight;

  if (top < list.scrollTop || bottom > list.scrollTop + list.clientHeight) {
    list.scrollTop = top - list.clientHeight / 2;
  }
}
