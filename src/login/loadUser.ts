import { LoaderFunctionArgs, RouterContextProvider } from "react-router";

import { userContext } from "./authMiddleware";

export default function loadUser({
  context,
}: LoaderFunctionArgs<RouterContextProvider>) {
  return context.get(userContext);
}
