import { Outlet, useLoaderData } from "react-router";

import { User } from "./UserContext";

export default function UserLayout() {
  const user = useLoaderData() as string;
  return (
    <User.Provider value={user}>
      <Outlet />
    </User.Provider>
  );
}
