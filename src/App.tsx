import { createBrowserRouter, RouterProvider } from "react-router";
import Home, { joinAction } from "./Home";
import Login, { loadUsername, loginAction, LoginRequired } from "./Login";
import Game, { loadGame } from "./Game";
import { ErrorBoundary } from "./Error";

const router = createBrowserRouter([
  {
    errorElement: <ErrorBoundary />,
    children: [
      {
        path: "/login",
        action: loginAction,
        element: <Login />,
      },
      {
        element: <LoginRequired />,
        loader: loadUsername,
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
