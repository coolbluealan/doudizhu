import { useEffect, useState, createContext } from "react";
import {
  ClientMsg,
  GameContextType,
  LobbyState,
  Msg,
  ServerMsg,
} from "./types";
import Chat from "./Chat";
import Hand, { CardList } from "./Hand";
import { Form, useLoaderData, useParams } from "react-router";
import { LoaderFunctionArgs } from "react-router";
import { fetchJson, FormError } from "./Error";

const cards: CardList = [
  [0, "3", "clubs"],
  [1, "5", "hearts"],
  [2, "7", "diamonds"],
  [3, "10", "spades"],
  [4, "SJ", "clubs"],
  [5, "BJ", "hearts"],
];

export const GameContext = createContext<GameContextType>({
  status: "Lobby",
  players: [],
  idx: null,
  messages: [],
  sendMessage: () => false,
  setMessages: () => {},
} as GameContextType);

export async function loadGame({ params }: LoaderFunctionArgs) {
  const lobbyCode = params.lobbyCode;
  return await Promise.all([
    fetchJson(`/api/lobby/${lobbyCode}`),
    fetchJson(`/api/lobby/${lobbyCode}/chat`),
  ]);
}

export default function Game() {
  const { lobbyCode } = useParams();
  const [lobbyState, initialMessages] = useLoaderData() as [LobbyState, Msg[]];

  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [messages, setMessages] = useState<Msg[]>(initialMessages);

  useEffect(() => {
    const protocol = window.location.protocol === "https:" ? "wss" : "ws";
    const ws = new WebSocket(
      `${protocol}://${window.location.host}/api/lobby/${lobbyCode}/ws`,
    );
    setSocket(ws);

    ws.onmessage = (e) => {
      const data: ServerMsg = JSON.parse(e.data);
      if ("Chat" in data) {
        setMessages((prev) => [...prev, data.Chat]);
      } else if ("State" in data) {
        // TODO
      }
    };

    return () => {
      ws.close();
    };
  }, [lobbyCode]);

  function sendMessage(msg: String): boolean {
    const message = msg.trim();
    if (msg && socket) {
      socket.send(
        JSON.stringify({
          Chat: message,
        } as ClientMsg),
      );
      return true;
    }
    return false;
  }

  return (
    <GameContext.Provider
      value={{ ...lobbyState, messages, setMessages, sendMessage }}
    >
      {lobbyState.idx == null && lobbyState.status == "Lobby" && (
        <Form action="/" method="POST">
          <input type="hidden" name="lobbyCode" value={lobbyCode} />
          <button type="submit">Join Game</button>
          <FormError />
        </Form>
      )}
      <Hand hand={cards} />
      <Chat />
    </GameContext.Provider>
  );
}
