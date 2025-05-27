import Card from "./Card";
import "./hand.css";

type HandProps = {
  hand: number[];
};
export default function Hand({ hand }: HandProps) {
  return (
    <div className="hand">
      {hand.map((card) => (
        <Card key={card} card={card} />
      ))}
    </div>
  );
}
