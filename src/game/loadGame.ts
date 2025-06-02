import { LoaderFunctionArgs } from "react-router";

import fetchJson from "@/fetchJson";

export async function loadGame({ params }: LoaderFunctionArgs) {
  const lobbyCode = params.lobbyCode as string;
  document.title = `Lobby ${lobbyCode}`;
  return await Promise.all([
    fetchJson(`/api/lobby/${lobbyCode}`),
    fetchJson(`/api/lobby/${lobbyCode}/chat`),
  ]);
}
