import "./info.css";

import { Fragment } from "react";
import { useParams } from "react-router";

import useGame from "../GameContext";

export default function Info() {
  const { lobbyCode } = useParams();
  const { players, game } = useGame();

  return (
    <div className="game-info overlay">
      <h1>
        <span className="lobby-code">{lobbyCode}</span>
      </h1>
      <div className="info-scores">
        <h3>SCOREBOARD</h3>
        <div className="info-players">
          {players.map(({ name, score }, idx) => (
            <Fragment key={idx}>
              <span className={`player-icon player-${idx}`} />
              <span className="medium">{name}</span>
              <span
                className={`player-score${
                  score == 0
                    ? ""
                    : " score-" + (score > 0 ? "positive" : "negative")
                }`}
              >
                {score}
              </span>
            </Fragment>
          ))}
        </div>
      </div>
      {game && "landlord" in game && (
        <div>
          <h3>GAME INFO</h3>
          <div className="info-state medium">
            <span>Bid:</span>
            <span>{game.bid}</span>
            <span>Multiplier:</span>
            <span>{game.mult}x</span>
            <span>Bonus:</span>
            <span>{game.bonus!}</span>
          </div>
        </div>
      )}
    </div>
  );
}
