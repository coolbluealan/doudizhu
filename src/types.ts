export type AppError = {
  status: number;
  statusText: string;
  msg: string;
};

export type ClientMsg =
  | { Chat: string }
  | { Start: null }
  | { Bid: number }
  | { Play: number[] };

export type Msg = {
  text: string;
  idx: number;
  time: number;
};

export type ServerMsg =
  | { Chat: Msg }
  | { State: LobbyState }
  | { Error: string };

export type GameState = {
  turn: number;
  bid: number;
  mult: number;
  passes: number;
  cards_left: number[];
  last_idx: number;
  last_play?: {
    kind: string;
    cards: number[];
  };
  landlord?: number;
  bonus?: number[];
  winner?: number;
};

export type Player = {
  name: string;
  score: number;
};

export type LobbyState = {
  status: "Lobby" | "Bidding" | "Playing" | "Finished";
  players: Player[];
  idx?: number;
  hand?: number[];
  game?: GameState;
};

export type GameContextType = LobbyState & {
  socket: WebSocket | null;
};
