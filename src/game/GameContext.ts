import { createContext, useContext } from "react";

import { GameContextType } from "@/types";

export const GameContext = createContext<GameContextType>({
  status: "Lobby",
  players: [],
  socket: null,
} satisfies GameContextType);

export default function useGame() {
  return useContext(GameContext);
}
