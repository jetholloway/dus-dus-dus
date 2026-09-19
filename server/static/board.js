// Drawing a board position. Shared by the play and replay views.

import { NAMES } from "./api.js";

const FILES = "ABCDEFG";

// Draws `state` into `container`. With `onSquare`, each square is a button
// that calls it with the square's name, such as "D3".
export function drawBoard(
  container,
  state,
  {
    onSquare = null,
    selected = null,
    targets = new Map(),
    movable = new Set(),
    last = new Set(),
  } = {},
) {
  container.replaceChildren();

  for (let rank = 7; rank >= 1; rank--) {
    container.append(label(rank, "rank"));

    for (const file of FILES) {
      const square = `${file}${rank}`;
      const owner = state.pieces[square];
      const cell = document.createElement(onSquare ? "button" : "div");
      cell.className = "square";

      if (onSquare) {
        cell.type = "button";
        cell.addEventListener("click", () => onSquare(square));
      } else {
        cell.setAttribute("role", "img");
      }

      if (owner) {
        const piece = document.createElement("span");
        piece.className = `piece ${owner.toLowerCase()}`;
        piece.classList.toggle("has-ball", state.ball === square);
        cell.append(piece);
      }

      cell.classList.toggle("selected", square === selected);
      cell.classList.toggle("movable", movable.has(square));
      cell.classList.toggle("last", last.has(square));
      if (targets.has(square)) {
        cell.classList.add(targets.get(square));
      }

      cell.setAttribute("aria-label", describe(state, square, owner));
      container.append(cell);
    }
  }

  container.append(label("", "file"));
  for (const file of FILES) {
    container.append(label(file, "file"));
  }
}

// The two squares an action such as "MOVE A1 A3" involves.
export function actionSquares(notation) {
  const [, src, dst] = notation.split(" ");
  return new Set([src, dst]);
}

function describe(state, square, owner) {
  if (!owner) {
    return `${square}, empty`;
  }
  const ball = state.ball === square ? ", holding the ball" : "";
  return `${square}, ${NAMES[owner]} piece${ball}`;
}

function label(text, kind) {
  const span = document.createElement("span");
  span.className = `label ${kind}`;
  span.textContent = text;
  return span;
}
