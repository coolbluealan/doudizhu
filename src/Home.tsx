import "./home.css";

import { useEffect } from "react";
import { Form, useNavigate } from "react-router";

import fetchJson from "./fetchJson";
import FormError from "./FormError";
import useUser from "./login/UserContext";

export default function Home() {
  const user = useUser();
  const navigate = useNavigate();

  useEffect(() => {
    document.title = "Home";
  }, []);

  async function handleCreate() {
    const { lobbyCode } = await fetchJson("/api/create", {
      method: "POST",
      credentials: "include",
    });
    navigate(`/lobby/${lobbyCode}`);
  }

  return (
    <div className="page">
      <nav>
        <b>Your username: {user}</b>
        <Form action="/logout" method="POST">
          <button className="btn-secondary" type="submit">
            Logout
          </button>
        </Form>
      </nav>

      <div className="center-container">
        <h1>
          <span className="title">Landlord / Doudizhu</span>
        </h1>
        <p>
          Join an existing lobby or create a new lobby.
          <br />
          Learn about the game{" "}
          <a href="https://en.wikipedia.org/wiki/Dou_dizhu">here</a>.
        </p>
        <Form method="POST">
          <div className="form-row">
            <input
              type="text"
              name="lobbyCode"
              maxLength={4}
              placeholder="Enter four letter lobby code"
              required
              onInput={(e) => {
                e.currentTarget.value = e.currentTarget.value.toUpperCase();
              }}
            />
            <button className="btn-primary" type="submit">
              Join Lobby
            </button>
          </div>
          <FormError />
        </Form>
        <hr className="separator" />
        <button className="create-btn btn-primary" onClick={handleCreate}>
          Create A New Lobby
        </button>
      </div>
    </div>
  );
}
