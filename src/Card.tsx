import { useState } from "react";
import "./card.css";

const rank_map = [
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
  return rank_map[Math.floor(card / 4)];
}

const suit_map = ["clubs", "diamonds", "hearts", "spades"];
function suit(card: number) {
  return suit_map[card % 4];
}
function color(card: number) {
  const suit_idx = card % 4;
  return suit_idx == 1 || suit_idx == 2 ? "red" : "black";
}

type CardProps = {
  card: number;
};
export default function Card({ card }: CardProps) {
  const [selected, setSelected] = useState(false);

  const c = color(card);
  const r = rank(card);
  const s = suit(card);

  return (
    <div
      className={`card card-${c}` + (selected ? " card-selected" : "")}
      onClick={() => {
        setSelected(!selected);
      }}
    >
      <div className="card-info">
        <b className="card-rank">{r}</b>
        <svg className="card-suit">
          <use href={`#${s}`} />
        </svg>
      </div>
    </div>
  );
}
