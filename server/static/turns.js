// Which actions to draw as arrows on the board.
//
// A turn is a run of consecutive actions by one side: three normally, one in
// setup. The move history already says who made each action, so the turns
// can be found from it alone.

const OTHER = { First: "Second", Second: "First" };

// Splits the history into turns: [{ player, actions: ["MOVE A1 A3", ...] }].
function turns(history) {
  const runs = [];

  for (const { player, action } of history) {
    const last = runs.at(-1);
    if (last && last.player === player) {
      last.actions.push(action);
    } else {
      runs.push({ player, actions: [action] });
    }
  }

  return runs;
}

// The arrows for one turn, numbered in the order its actions were played
// unless `faint`, which marks the player's own actions this turn.
function arrowsFor(turn, { faint = false } = {}) {
  return turn.actions.map((action, index) => {
    const [type, from, to] = action.split(" ");
    return {
      kind: type.toLowerCase(),
      from,
      to,
      side: turn.player,
      label: faint ? null : String(index + 1),
      faint,
    };
  });
}

// For the play view: the opponent's most recent turn, including one still in
// progress, and the viewer's own actions so far this turn, drawn faintly.
//
// The opponent is the side the viewer doesn't play. In hot-seat, where they
// play both, and when watching, it's the side that isn't about to move. Once
// the game is won, only the winning turn is drawn.
export function playArrows(game) {
  const runs = turns(game.history);
  const last = runs.at(-1);

  if (last === undefined) {
    return [];
  }

  if (game.state.winner !== null) {
    return arrowsFor(last);
  }

  const toMove = game.state.current_player;
  const mine = Object.keys(game.seats).filter((side) => game.seats[side].is_you);
  const opponent = mine.length === 1 ? OTHER[mine[0]] : OTHER[toMove];

  const theirs = runs.findLast((run) => run.player === opponent);
  const own = last.player === toMove && toMove !== opponent ? last : null;

  return [
    ...(own ? arrowsFor(own, { faint: true }) : []),
    ...(theirs ? arrowsFor(theirs) : []),
  ];
}

// For the replay view: the turn containing move `frame`, up to that move, so
// stepping through draws each turn as it happened.
export function replayArrows(history, frame) {
  const last = turns(history.slice(0, frame)).at(-1);
  return last === undefined ? [] : arrowsFor(last);
}
