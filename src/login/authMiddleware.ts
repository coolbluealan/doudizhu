import {
  redirect,
  unstable_createContext,
  unstable_MiddlewareFunction,
} from "react-router";

// empty string means logged out
export const userContext = unstable_createContext<string>("");

const authMiddleware: unstable_MiddlewareFunction = async ({
  request,
  context,
}) => {
  // authenticated
  if (context.get(userContext)) return;

  const url = new URL(request.url);
  const relative = url.pathname + url.search + url.hash;

  const resp = await fetch("/api/me", { credentials: "include" });
  if (!resp.ok) {
    throw redirect(relative == "/" ? "/login" : `/login?next=${relative}`);
  }

  const json = await resp.json();
  context.set(userContext, json.username);
};
export default authMiddleware;
