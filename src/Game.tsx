import { useEffect, useState } from "react";
import Chat from "./Chat";
import Hand, { CardList } from "./Hand";

const URL = "wss://" + window.location.hostname + "/api/ws";

const cards: CardList = [
  [0, "3", "clubs"],
  [1, "5", "hearts"],
  [2, "7", "diamonds"],
  [3, "10", "spades"],
  [4, "SJ", "clubs"],
  [5, "BJ", "hearts"],
];

export default function Game() {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  useEffect(() => {
    const ws = new WebSocket(URL);
    setSocket(ws);
    socket;
  }, []);

  return (
    <>
      <Hand hand={cards} />
      <Chat />
    </>
  );
}
