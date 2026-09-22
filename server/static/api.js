// Talking to the server, and the names the UI shows for things.

export const NAMES = { First: "Orange", Second: "Teal" };

export const MODES = { bot: "vs bot", hotseat: "hot-seat", online: "online" };

// This browser's identity. There are no accounts: a random id says which
// browser you are, and a name you choose is shown beside the seats you take.
// Nothing is checked, so players are trusted to name themselves honestly.
const ID_KEY = "dus.player.id";
const NAME_KEY = "dus.player.name";

export function playerId() {
  let id = localStorage.getItem(ID_KEY);

  if (id === null) {
    id = crypto.randomUUID();
    localStorage.setItem(ID_KEY, id);
  }

  return id;
}

export function playerName() {
  return localStorage.getItem(NAME_KEY) ?? "";
}

export function setPlayerName(name) {
  localStorage.setItem(NAME_KEY, name.trim().slice(0, 40));
}

export async function api(path, options = {}) {
  const response = await fetch(`/api${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      "X-Player": playerId(),
      ...options.headers,
    },
  });
  const body = await response.json();

  if (!response.ok) {
    const error = new Error(body.detail ?? response.statusText);
    error.status = response.status;
    throw error;
  }

  return body;
}

export function setMessage(element, text, isError = false) {
  element.textContent = text;
  element.classList.toggle("error", isError);
}
