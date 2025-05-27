import {
  ActionFunctionArgs,
  Form,
  LoaderFunctionArgs,
  Outlet,
  redirect,
  unstable_createContext,
  unstable_MiddlewareFunction,
  unstable_RouterContextProvider,
  useLoaderData,
  useSearchParams,
} from "react-router";
import { FormError } from "./Error";

// empty string means logged out
const userContext = unstable_createContext<string>("");

export const authMiddleware: unstable_MiddlewareFunction = async ({
  request,
  context,
}) => {
  // already authenticated
  if (context.get(userContext)) {
    return;
  }

  const url = new URL(request.url);
  const relative = url.pathname + url.search + url.hash;

  const resp = await fetch("/api/me", { credentials: "include" });
  if (!resp.ok) {
    throw redirect(relative == "/" ? "/login" : `/login?next=${relative}`);
  }

  const json = await resp.json();
  context.set(userContext, json.username);
};

export function loadUser({
  context,
}: LoaderFunctionArgs<unstable_RouterContextProvider>) {
  return context.get(userContext);
}

export function LoginRequired() {
  const user = useLoaderData() as string;
  return (
    <>
      <nav>
        {user}
        <Form action="/logout" method="POST">
          <button type="submit">Logout</button>
        </Form>
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

export async function logoutAction({
  context,
}: LoaderFunctionArgs<unstable_RouterContextProvider>) {
  context.set(userContext, "");
  await fetch("/api/logout", {
    method: "POST",
    credentials: "include",
  });
  return redirect("/login");
}
