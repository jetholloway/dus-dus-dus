// The games view: every saved game, most recently played first.

import { MODES, NAMES, api, setMessage } from "./api.js";

const element = (id) => document.getElementById(id);

export async function showGames() {
  const body = element("games-body");
  body.replaceChildren();
  setMessage(element("games-message"), "");

  let games;
  try {
    games = await api("/games");
  } catch (error) {
    setMessage(element("games-message"), error.message, true);
    return;
  }

  element("games-table").hidden = games.length === 0;
  element("games-empty").hidden = games.length !== 0;

  for (const game of games) {
    body.append(row(game));
  }
}

function row(game) {
  const tr = document.createElement("tr");

  tr.append(
    cell(formatDate(game.created_at)),
    cell(MODES[game.mode]),
    cell(players(game)),
    cell(String(game.moves)),
    resultCell(game),
    linksCell(game),
  );

  return tr;
}

// "Jet vs Bot", with a dash for a side nobody has claimed.
function players(game) {
  const name = (side) => game.seats[side].name || "\u2014";
  return `${name("First")} vs ${name("Second")}`;
}

function resultCell(game) {
  if (game.status === "unreplayable") {
    const td = cell("Older rules");
    td.title = game.problem;
    td.className = "problem";
    return td;
  }
  if (game.status === "finished") {
    return cell(`${winnerName(game)} won`);
  }
  return cell("In progress");
}

// The winner's name, such as "Jet". The colour stands in for a seat with no
// name, from before seats were recorded, and is added when one person played
// both sides, as in hot-seat: "Jet (Orange)".
function winnerName(game) {
  const colour = NAMES[game.winner];
  const name = game.seats[game.winner].name;
  const loser = game.seats[game.winner === "First" ? "Second" : "First"].name;

  if (!name) {
    return colour;
  }
  return name === loser ? `${name} (${colour})` : name;
}

function linksCell(game) {
  const td = document.createElement("td");
  td.className = "links";
  td.append(link("Replay", `#replay/${game.id}`));
  if (game.status === "in_progress") {
    td.append(link("Continue", `#play/${game.id}`));
  }
  return td;
}

function cell(text) {
  const td = document.createElement("td");
  td.textContent = text;
  return td;
}

function link(text, href) {
  const a = document.createElement("a");
  a.textContent = text;
  a.href = href;
  return a;
}

function formatDate(iso) {
  return new Date(iso).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}
