import { useState } from "react";
import "./card.css";

export type Rank =
  | "3"
  | "4"
  | "5"
  | "6"
  | "7"
  | "8"
  | "9"
  | "10"
  | "J"
  | "Q"
  | "K"
  | "A"
  | "2"
  | "SJ"
  | "BJ";

export type Suit = "clubs" | "diamonds" | "spades" | "hearts";
const get_symbol: Record<Suit, string> = {
  hearts: "♥",
  diamonds: "♦",
  clubs: "♣",
  spades: "♠",
};

type CardProps = {
  rank: Rank;
  suit: Suit;
};

export default function Card({ rank, suit }: CardProps) {
  const color = suit == "spades" ? "black" : "red";
  const symbol = (function () {
    switch (rank) {
      case "SJ":
        return "SM";
      case "BJ":
        return "BIG";
      default:
        return get_symbol[suit];
    }
  })();

  const [selected, setSelected] = useState(false);

  return (
    <div
      className={`card card-${color}` + (selected ? " selected" : "")}
      onClick={() => {
        setSelected(!selected);
      }}
    >
      <div className="card-rank">{rank}</div>
      <div className="card-suit">{symbol}</div>
    </div>
  );
}
