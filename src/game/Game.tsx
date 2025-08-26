import "./game.css";

import { useEffect, useState } from "react";
import { useLoaderData, useParams } from "react-router";

import { LobbyState, Msg, ServerMsg } from "@/types";

import SuitDefs from "./card/SuitDefs";
import Chat from "./chat/Chat";
import { GameContext } from "./GameContext";
import Hand from "./hand/Hand";
import Info from "./info/Info";
import Table from "./table/Table";

export default function Game() {
  const { lobbyCode } = useParams();
  const [initialLobbyState, initialMessages] =
    useLoaderData<[LobbyState, Msg[]]>();

  const [lobbyState, setLobbyState] = useState(initialLobbyState);
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [error, setError] = useState("");
  const [reconnect, setReconnect] = useState(false); // toggle to trigger reconnect

  // set state if the loader multiple times
  // this happens on the transition from spectator to user
  useEffect(() => {
    setLobbyState(initialLobbyState);
  }, [initialLobbyState]);

  // connect websocket
  useEffect(() => {
    const protocol = window.location.protocol == "https:" ? "wss" : "ws";
    const ws = new WebSocket(
      `${protocol}://${window.location.host}/api/lobby/${lobbyCode}/ws`,
    );
    setSocket(ws);
    setError("");

    // handle game state update messages
    ws.onmessage = (e: MessageEvent) => {
      const data: ServerMsg = JSON.parse(e.data);
      if ("State" in data) {
        setLobbyState(data.State);
      } else if ("Error" in data) {
        setError(data.Error);
      }
    };

    // handle reconnection
    ws.onclose = () => {
      setTimeout(() => {
        setError("reconnecting...");
        setReconnect(!reconnect);
      }, 1000);
    };

    return () => {
      ws.close();
    };
  }, [lobbyCode, lobbyState.idx, reconnect]);

  // generate notification
  let msg;
  if (error) {
    msg = `Error: ${error}`;
  } else if (lobbyState.status == "Bidding" || lobbyState.status == "Playing") {
    msg = `${
      lobbyState.game!.turn == lobbyState.idx
        ? "Your"
        : `${lobbyState.players[lobbyState.game!.turn].name}'s`
    } turn`;
  }

  return (
    <GameContext.Provider value={{ ...lobbyState, socket }}>
      <div className="game">
        <SuitDefs />
        <Info />
        {msg && <div className="game-notification hover-box medium">{msg}</div>}
        <Table />
        <Chat initial={initialMessages} />
        <Hand hand={lobbyState.hand || []} />
      </div>
    </GameContext.Provider>
  );
}
