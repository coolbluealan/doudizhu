import { createContext, useEffect, useState } from "react";
import {
  Form,
  LoaderFunctionArgs,
  useLoaderData,
  useParams,
} from "react-router";
import { fetchJson, FormError } from "./Error";
import Chat from "./Chat";
import Hand from "./Hand";
import SuitDefs from "./SuitDefs";
import { GameContextType, LobbyState, Msg, ServerMsg } from "./types";

export const GameContext = createContext<GameContextType>({
  status: "Lobby",
  players: [],
  idx: null,
  socket: null,
} satisfies GameContextType);

export async function loadGame({ params }: LoaderFunctionArgs) {
  const lobbyCode = params.lobbyCode;
  return await Promise.all([
    fetchJson(`/api/lobby/${lobbyCode}`),
    fetchJson(`/api/lobby/${lobbyCode}/chat`),
  ]);
}

export default function Game() {
  const { lobbyCode } = useParams();
  const [lobbyState, initialMessages] = useLoaderData<[LobbyState, Msg[]]>();

  const [socket, setSocket] = useState<WebSocket | null>(null);

  useEffect(() => {
    const protocol = window.location.protocol == "https:" ? "wss" : "ws";
    const ws = new WebSocket(
      `${protocol}://${window.location.host}/api/lobby/${lobbyCode}/ws`,
    );
    setSocket(ws);

    ws.addEventListener("message", (e: MessageEvent) => {
      const data: ServerMsg = JSON.parse(e.data);
      if ("State" in data) {
        // TODO
      }
    });

    return () => {
      ws.close();
    };
  }, [lobbyCode]);

  return (
    <GameContext.Provider value={{ ...lobbyState, socket }}>
      {lobbyState.idx == null && lobbyState.status == "Lobby" && (
        <Form action="/" method="POST">
          <input type="hidden" name="lobbyCode" value={lobbyCode} />
          <button type="submit">Join Game</button>
          <FormError />
        </Form>
      )}
      <SuitDefs />
      <Hand hand={[0, 5, 10, 15, 24, 29, 34, 39, 44, 49, 52, 53]} />
      <Chat initial={initialMessages} />
    </GameContext.Provider>
  );
}
