import "./table.css";

import PassIndicator from "@/assets/pass-indicator.svg?react";
import TurnIndicator from "@/assets/turn-indicator.svg?react";

import Card from "../card/Card";
import useGame from "../GameContext";

export default function Table() {
  const { status, players, game } = useGame();

  let text;
  let field;
  let emph;
  if (status == "Lobby") {
    if (players.length < 3) {
      text = "Waiting for players...";
    } else {
      text = "Ready to start.";
    }
  } else if (status == "Bidding") {
    text = "Bidding phase.";
    emph = `Current bid: ${game!.bid || "none"}`;
  } else {
    const last_play = game!.last_play!;
    text = last_play.cards.length
      ? `${players[game!.last_idx].name} played`
      : "New round";
    field = (
      <div className="table-field cards">
        {last_play.cards.map((card) => (
          <Card key={card} card={card} />
        ))}
      </div>
    );

    if (status == "Finished") {
      emph =
        game!.winner == game!.landlord ? "Landlord wins!" : "Peasants win!";
    } else {
      emph = last_play.cards.length ? `${last_play.kind}` : "";
    }
  }

  return (
    <div className="game-table">
      <Player pos={0} />
      <Player pos={1} />
      <Player pos={2} />
      <div className="table-center">
        {text && <span className="medium">{text}</span>}
        {field}
        {emph && <span className="table-emph medium">{emph}</span>}
      </div>
    </div>
  );
}

type PlayerProps = {
  pos: number;
};
function Player({ pos }: PlayerProps) {
  const { players, idx, game } = useGame();

  const i = (pos + (idx || 0)) % 3;
  if (players.length <= i) {
    return null;
  }

  let indicator;
  if (game) {
    if (i == game.turn) {
      indicator = <TurnIndicator className="turn-indicator" />;
    } else if ((game.turn + 3 - i) % 3 <= game.passes) {
      indicator = <PassIndicator className="pass-indicator" />;
    }
  }

  return (
    <div className={`table-player-${pos} hover-box`}>
      <div className="player-name">
        <div className="indicator">{indicator}</div>
        <b className={`player-${i}`}>{players[i].name}</b>{" "}
        {i == game?.landlord && (
          <span className="landlord-icon medium">地主</span>
        )}
      </div>
      {game && (
        <div
          className={`cards-left medium${
            game.cards_left[i] < 8
              ? " cards-" + (game.cards_left[i] > 0 ? "low" : "zero")
              : ""
          }`}
        >
          {game.cards_left[i]} card{game.cards_left[i] == 1 ? "" : "s"}
        </div>
      )}
    </div>
  );
}
