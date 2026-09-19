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
    cell(String(game.moves)),
    resultCell(game),
    linksCell(game),
  );

  return tr;
}

function resultCell(game) {
  if (game.status === "unreplayable") {
    const td = cell("Older rules");
    td.title = game.problem;
    td.className = "problem";
    return td;
  }
  if (game.status === "finished") {
    return cell(`${NAMES[game.winner]} won`);
  }
  return cell("In progress");
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
