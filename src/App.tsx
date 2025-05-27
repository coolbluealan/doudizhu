import { createBrowserRouter, RouterProvider } from "react-router";
import Home, { joinAction } from "./Home";
import Login, {
  authMiddleware,
  loadUser,
  loginAction,
  LoginRequired,
  logoutAction,
} from "./Login";
import Game, { loadGame } from "./Game";
import { ErrorBoundary } from "./Error";
import "./global.css";

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
        path: "/logout",
        action: logoutAction,
      },
      {
        element: <LoginRequired />,
        unstable_middleware: [authMiddleware],
        loader: loadUser,
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
