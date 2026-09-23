// Drawing a board position. Shared by the play and replay views.

import { NAMES } from "./api.js";

const FILES = "ABCDEFG";
const SVG = "http://www.w3.org/2000/svg";

// Draws `state` into `container`. With `onSquare`, each square is a button
// that calls it with the square's name, such as "D3". `arrows` come from
// turns.js and show recent actions.
export function drawBoard(
  container,
  state,
  {
    onSquare = null,
    selected = null,
    targets = new Map(),
    movable = new Set(),
    arrows = [],
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

  if (arrows.length > 0) {
    container.append(drawArrows(container.id, arrows));
  }
}

// Arrows: a drawing laid exactly over the 7x7 squares, in units of one square,
// so it lines up however large the board is drawn. It ignores the mouse, so
// clicks reach the squares beneath. The move list says the same in words, so
// screen readers skip it.
function drawArrows(boardId, arrows) {
  const svg = svgElement("svg", { class: "arrows", viewBox: "0 0 7 7", "aria-hidden": "true" });
  const defs = svgElement("defs");

  // Arrowheads are markers, one per side's colour. Their ids include the
  // board's, since the play and replay boards share the page.
  for (const side of ["first", "second"]) {
    // refX near 10 puts the tip at the end of the line, not beyond it.
    const marker = svgElement("marker", {
      id: `${boardId}-arrowhead-${side}`,
      viewBox: "0 0 10 10",
      refX: "9",
      refY: "5",
      markerWidth: "2.8",
      markerHeight: "2.8",
      orient: "auto",
    });
    marker.append(svgElement("path", { d: "M 0 0 L 10 5 L 0 10 z", class: `arrowhead ${side}` }));
    defs.append(marker);
  }
  svg.append(defs);

  for (const arrow of arrows) {
    svg.append(drawArrow(boardId, arrow));
  }

  return svg;
}

function drawArrow(boardId, { kind, from, to, side, label, faint }) {
  const [x1, y1] = centre(from);
  const [x2, y2] = centre(to);

  // End at the edge of the piece rather than its centre, so the arrow shows up
  // against the board instead of vanishing over a piece of the same colour. A
  // piece reaches about 0.36 of a square from the centre. A move or tackle
  // leaves its first square empty, so its arrow can start nearly at the centre,
  // which keeps a one-square step long enough to see; a pass starts at the edge
  // of the piece that threw it.
  const length = Math.hypot(x2 - x1, y2 - y1);
  const [ux, uy] = [(x2 - x1) / length, (y2 - y1) / length];
  const start = kind === "pass" ? 0.34 : 0.12;
  const [sx, sy] = [x1 + ux * start, y1 + uy * start];
  const [ex, ey] = [x2 - ux * 0.4, y2 - uy * 0.4];

  const sideClass = side.toLowerCase();
  const group = svgElement("g", {
    class: `arrow ${sideClass} ${kind}${faint ? " faint" : ""}`,
  });

  group.append(
    svgElement("line", {
      x1: sx,
      y1: sy,
      x2: ex,
      y2: ey,
      "marker-end": `url(#${boardId}-arrowhead-${sideClass})`,
    }),
  );

  // A tackle's arrow is the tackler's step; the ring says it took the ball.
  if (kind === "tackle") {
    group.append(svgElement("circle", { class: "took-ball", cx: x2, cy: y2, r: "0.47" }));
  }

  // The number sits beside the arrow's middle, not on it: an arrow between
  // neighbouring squares is short enough for a number to hide it.
  if (label !== null) {
    const [mx, my] = [(sx + ex) / 2 - uy * 0.2, (sy + ey) / 2 + ux * 0.2];
    group.append(svgElement("circle", { class: "number", cx: mx, cy: my, r: "0.15" }));
    const text = svgElement("text", { x: mx, y: my });
    text.textContent = label;
    group.append(text);
  }

  return group;
}

// The centre of a square such as "D3", in square units from the top left.
function centre(square) {
  const file = FILES.indexOf(square[0]);
  const rank = Number(square.slice(1));
  return [file + 0.5, 7 - rank + 0.5];
}

function svgElement(name, attributes = {}) {
  const element = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attributes)) {
    element.setAttribute(key, value);
  }
  return element;
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
