import "./card.css";

import { MouseEventHandler } from "react";

import joker from "@/assets/joker.png";

const rankMap = [
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "10",
  "J",
  "Q",
  "K",
  "A",
  "2",
  "J",
];
function rank(card: number) {
  return rankMap[Math.floor(card / 4)];
}

const suitMap = ["clubs", "diamonds", "hearts", "spades"];
function suit(card: number) {
  if (card >= 52) return "joker";
  return suitMap[card % 4];
}

const colorMap = ["black", "red", "red", "black"];
function color(card: number) {
  return colorMap[card % 4];
}

type CardProps = {
  card: number;
  small?: boolean;
  selected?: boolean;
  onMouseDown?: () => void;
  onMouseEnter?: MouseEventHandler;
};
export default function Card({
  card,
  small = false,
  selected = false,
  onMouseDown,
  onMouseEnter,
}: CardProps) {
  const c = color(card);
  const r = rank(card);
  const s = suit(card);

  return (
    <div
      className={
        `card card-${c}` +
        (small ? " card-small" : "") +
        (selected ? " card-selected" : "")
      }
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
