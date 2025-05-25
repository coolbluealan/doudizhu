import { ActionFunctionArgs, Form, redirect, useNavigate } from "react-router";
import { fetchJson, FormError } from "./Error";

export async function joinAction({ request }: ActionFunctionArgs) {
  const data = await request.formData();
  const lobbyCode = (data.get("lobbyCode") as string).trim().toUpperCase();

  // validate username
  if (!lobbyCode) {
    return { error: "Lobby code cannot be empty" };
  }

  const resp = await fetch(`/api/lobby/${lobbyCode}/join`, {
    method: "POST",
    credentials: "include",
  });

  if (!resp.ok) {
    const error = await resp.json();
    return { error: error.msg || `Failed to join ${lobbyCode}` };
  }

  return redirect(`/lobby/${lobbyCode}`);
}

export default function Home() {
  const navigate = useNavigate();

  async function handleCreate() {
    const { lobbyCode } = await fetchJson("/api/create", {
      method: "POST",
      credentials: "include",
    });
    navigate(`/lobby/${lobbyCode}`);
  }

  return (
    <>
      <h1>Welcome</h1>
      <button onClick={handleCreate}>Create Lobby</button>
      <Form method="POST">
        <input
          type="text"
          name="lobbyCode"
          maxLength={4}
          placeholder="Enter four letter lobby code"
          required
        />
        <button type="submit">Join</button>
        <FormError />
      </Form>
    </>
  );
}
