// Talking to the server, and the names the UI shows for things.

export const NAMES = { First: "Orange", Second: "Teal" };

export const MODES = { bot: "vs bot", hotseat: "hot-seat" };

export async function api(path, options = {}) {
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

export function setMessage(element, text, isError = false) {
  element.textContent = text;
  element.classList.toggle("error", isError);
}
