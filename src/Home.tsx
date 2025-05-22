import { useNavigate } from "react-router";

export default function Home() {
  const navigate = useNavigate();

  async function handleCreate() {
    const resp = await fetch("/api/create", {
      method: "POST",
      credentials: "include",
    });

    const { lobby_code } = await resp.json();
    navigate(`/lobby/${lobby_code}`);
  }

  return (
    <>
      <h1>Welcome</h1>
      <button onClick={handleCreate}>Create Lobby</button>
    </>
  );
}
