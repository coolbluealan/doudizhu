import Card, { Rank, Suit } from "./Card";
import "./hand.css";

export type CardList = Array<[number, Rank, Suit]>;
type HandProps = {
  hand: CardList;
};
export default function Hand({ hand }: HandProps) {
  return (
    <div className="hand">
      {hand.map(([id, rank, suit]) => (
        <Card key={id} rank={rank} suit={suit} />
      ))}
    </div>
  );
}
