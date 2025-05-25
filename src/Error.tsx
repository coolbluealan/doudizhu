import { Link, useActionData, useRouteError } from "react-router";
import { AppError } from "./types";

export async function fetchJson(url: string, options: RequestInit = {}) {
  const resp = await fetch(url, options);

  const parseError = (data: any) =>
    ({
      status: resp.status,
      statusText: resp.statusText,
      msg: data.msg || "Unknown error",
    }) as AppError;

  return resp
    .json()
    .catch((e: Error) => {
      throw parseError(e.toString());
    })
    .then((json) => {
      if (!resp.ok) throw parseError(json.msg);
      return json;
    });
}

export function FormError() {
  const actionData = useActionData();
  if (!actionData?.error) {
    return null;
  }

  return <div>{actionData.error}</div>;
}

export function ErrorBoundary() {
  const error = useRouteError() as AppError;
  return (
    <>
      <h1>{error.status}</h1>
      <h2>{error.statusText}</h2>
      <div>{error.msg}</div>
      <Link to="/">Return home</Link>
    </>
  );
}
