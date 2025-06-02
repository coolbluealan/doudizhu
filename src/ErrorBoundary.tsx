import { Link, useRouteError } from "react-router";

import { AppError } from "./types";

export default function ErrorBoundary() {
  const error = useRouteError() as AppError;
  return (
    <>
      <h1>{error.status}</h1>
      <h2>{error.statusText}</h2>
      <p>{error.msg}</p>
      <Link to="/">Return home</Link>
    </>
  );
}
