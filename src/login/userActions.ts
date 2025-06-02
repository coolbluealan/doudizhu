import {
  ActionFunctionArgs,
  redirect,
  unstable_RouterContextProvider,
} from "react-router";

import { userContext } from "./authMiddleware";

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
}: ActionFunctionArgs<unstable_RouterContextProvider>) {
  context.set(userContext, "");
  await fetch("/api/logout", {
    method: "POST",
    credentials: "include",
  });
  return redirect("/login");
}
