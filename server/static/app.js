// Browser client for the Dus Dus Dus server: picks a view from the address.
//
//   #               the list of saved games
//   #play/<id>      playing a game
//   #replay/<id>    stepping through a game
//
// A bare #<id>, from before there were views, opens the game to play.

import { playerName, setPlayerName } from "./api.js";
import { showGames } from "./games.js";
import { playKey, showPlay, startGame } from "./play.js";
import { replayKey, setUpReplayControls, showReplay, stopReplay } from "./replay.js";

const VIEWS = {
  games: { element: "games-view", show: () => showGames(), key: null },
  play: { element: "play-view", show: showPlay, key: playKey },
  replay: { element: "replay-view", show: showReplay, key: replayKey },
};

let current = null;

function parseHash() {
  const hash = location.hash.slice(1);

  if (hash === "") {
    return ["games", null];
  }

  const [view, id] = hash.split("/");
  return id === undefined ? ["play", view] : [view, id];
}

async function route() {
  const [name, id] = parseHash();

  if (!(name in VIEWS)) {
    location.hash = "";
    return;
  }

  stopReplay();
  current = VIEWS[name];

  for (const view of Object.values(VIEWS)) {
    document.getElementById(view.element).hidden = view !== current;
  }

  await current.show(id);
}

function start() {
  const name = document.getElementById("player-name");
  name.value = playerName();
  name.addEventListener("change", () => setPlayerName(name.value));

  document.getElementById("new-bot").addEventListener("click", () => startGame("bot"));
  document.getElementById("new-hotseat").addEventListener("click", () => startGame("hotseat"));
  setUpReplayControls();

  document.addEventListener("keydown", (event) => current?.key?.(event));
  window.addEventListener("hashchange", route);

  route();
}

start();
