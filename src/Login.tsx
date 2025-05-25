import {
  LoaderFunctionArgs,
  useLoaderData,
  useNavigate,
  useSearchParams,
} from "react-router";
import { ActionFunctionArgs, Form, Outlet, redirect } from "react-router";
import { FormError } from "./Error";

export async function loadUsername({ request }: LoaderFunctionArgs) {
  const url = new URL(request.url);
  const relative = url.pathname + url.search + url.hash;

  const resp = await fetch("/api/me", { credentials: "include" });
  if (!resp.ok) {
    return redirect(relative == "/" ? "/login" : `/login?next=${relative}`);
  }

  const data = await resp.json();
  return data.username;
}

export function LoginRequired() {
  const username = useLoaderData() as string;
  const navigate = useNavigate();

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

export default function Login() {
  const [searchParams] = useSearchParams();
  const to = searchParams.get("next") || "/";

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
        <FormError />
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
