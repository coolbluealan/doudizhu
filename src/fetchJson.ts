import { AppError } from "@/types";

export default async function fetchJson(
  url: string,
  options: RequestInit = {},
) {
  const resp = await fetch(url, options);

  const parseError = (data: { msg?: string }) =>
    ({
      status: resp.status,
      statusText: resp.statusText,
      msg: data.msg || "Unknown error",
    }) satisfies AppError;

  return resp
    .json()
    .catch((e: Error) => {
      throw parseError({ msg: e.toString() });
    })
    .then((json) => {
      if (!resp.ok) throw parseError(json.msg);
      return json;
    });
}
