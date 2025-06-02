import "./card.css";

import { MouseEventHandler } from "react";

import joker from "@/assets/joker.png";

import { color, rank, suit } from "./cardUtils";

type CardProps = {
  card: number;
  selected?: boolean;
  onMouseDown?: () => void;
  onMouseEnter?: MouseEventHandler;
};
export default function Card({
  card,
  selected = false,
  onMouseDown,
  onMouseEnter,
}: CardProps) {
  const c = color(card);
  const r = rank(card);
  const s = suit(card);

  return (
    <div
      className={`card card-${c}` + (selected ? " card-selected" : "")}
      onMouseDown={onMouseDown}
      onMouseEnter={onMouseEnter}
    >
      <div className="card-info">
        <b>{r}</b>
        {card < 52 && (
          <svg className="card-suit">
            <use href={`#${s}`} />
          </svg>
        )}
      </div>
      {card >= 52 && (
        <img
          className={`joker${card == 52 ? " small-joker" : ""}`}
          src={joker}
          alt="Joker"
          draggable={false}
        />
      )}
    </div>
  );
}
