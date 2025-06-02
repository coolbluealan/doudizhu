import {
  LoaderFunctionArgs,
  unstable_RouterContextProvider,
} from "react-router";

import { userContext } from "./authMiddleware";

export default function loadUser({
  context,
}: LoaderFunctionArgs<unstable_RouterContextProvider>) {
  return context.get(userContext);
}
