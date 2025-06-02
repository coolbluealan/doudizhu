import { ActionFunctionArgs, redirect } from "react-router";

export async function joinAction({ request }: ActionFunctionArgs) {
  const data = await request.formData();
  const lobbyCode = (data.get("lobbyCode") as string).trim().toUpperCase();

  // validate lobbyCode
  if (lobbyCode.length != 4) {
    return { error: "lobby code must be 4 letters" };
  }

  const resp = await fetch(`/api/lobby/${lobbyCode}/join`, {
    method: "POST",
    credentials: "include",
  });

  if (!resp.ok) {
    const error = await resp.json();
    return { error: error.msg || `failed to join ${lobbyCode}` };
  }

  return redirect(`/lobby/${lobbyCode}`);
}
