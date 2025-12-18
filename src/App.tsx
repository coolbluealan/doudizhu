import { createBrowserRouter, RouterProvider } from "react-router";

import ErrorBoundary from "./ErrorBoundary";
import Game from "./game/Game";
import { joinAction } from "./game/joinAction";
import { loadGame } from "./game/loadGame";
import Home from "./Home";
import authMiddleware from "./login/authMiddleware";
import loadUser from "./login/loadUser";
import Login from "./login/Login";
import { loginAction, logoutAction } from "./login/userActions";
import UserLayout from "./login/UserLayout";

const router = createBrowserRouter([
  {
    errorElement: <ErrorBoundary />,
    HydrateFallback: () => null,
    children: [
      {
        path: "/login",
        action: loginAction,
        element: <Login />,
      },
      {
        path: "/logout",
        action: logoutAction,
      },
      {
        middleware: [authMiddleware],
        loader: loadUser,
        element: <UserLayout />,
        children: [
          {
            path: "/",
            action: joinAction,
            element: <Home />,
          },
          {
            path: "/lobby/:lobbyCode",
            loader: loadGame,
            element: <Game />,
          },
        ],
      },
    ],
  },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
