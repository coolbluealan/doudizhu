import { useEffect, useState } from "react";
import { Navigate, useNavigate } from "react-router";
import {
  ActionFunctionArgs,
  Form,
  Outlet,
  redirect,
  useActionData,
  useLocation,
} from "react-router";

export function LoginRequired() {
  const [username, setUsername] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    (async () => {
      const resp = await fetch("/api/me", { credentials: "include" });
      if (resp.ok) {
        const data = await resp.json();
        setUsername(data.username);
      }
      setLoading(false);
    })();
  }, []);

  if (loading) {
    return <div>Loading...</div>;
  }
  if (username == null) {
    return (
      <Navigate
        to="/login"
        state={{ from: location.pathname + location.search + location.hash }}
      />
    );
  }

  async function handleLogout() {
    await fetch("/api/logout", {
      method: "POST",
      credentials: "include",
    });
    return navigate("/login");
  }

  return (
    <>
      <nav>
        {username}
        <button onClick={handleLogout}>Logout</button>
      </nav>
      <Outlet />
    </>
  );
}

export function Login() {
  const actionData = useActionData();
  const location = useLocation();
  const to = location.state?.from || "/";

  return (
    <>
      <h1>Login Page</h1>
      <Form method="POST">
        <div>
          <label htmlFor="username">Username</label>
          <input type="text" name="username" required />
        </div>
        <input type="hidden" name="to" value={to} />
        <button type="submit">Login</button>
        {actionData?.error && <div>{actionData.error}</div>}
      </Form>
    </>
  );
}

export async function loginAction({ request }: ActionFunctionArgs) {
  const data = await request.formData();
  const username = (data.get("username") as string).trim();
  const to = data.get("to") as string;

  // validate username
  if (!username) {
    return { error: "Username cannot be empty" };
  }

  const resp = await fetch("/api/login", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({ username }),
    credentials: "include",
  });
  if (!resp.ok) {
    return { error: "Login failed" };
  }

  return redirect(to);
}
