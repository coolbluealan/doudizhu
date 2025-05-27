export type AppError = {
  status: number;
  statusText: string;
  msg: string;
};

export type Cards = number[];

export type ClientMsg = { Chat: string } | { Move: Cards };

export type Msg = {
  text: string;
  idx: number;
  time: number;
};

export type ServerMsg = { Chat: Msg } | { State: LobbyState };

export type GameState = {
  turn: number;
  bid: number;
  mult?: number;
  landlord?: number;
  bonus?: number[];
  winner?: number;
};

export type Player = {
  name: string;
  score: number;
};

export type LobbyState = {
  status: string;
  players: Player[];
  idx: number | null;
  hand?: number[];
  game?: GameState;
};

export type GameContextType = LobbyState & {
  socket: WebSocket | null;
};
