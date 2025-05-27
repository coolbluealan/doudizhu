import { createBrowserRouter, RouterProvider } from "react-router";
import { ErrorBoundary } from "./Error";
import Game, { loadGame } from "./Game";
import Home, { joinAction } from "./Home";
import Login, {
  authMiddleware,
  loadUser,
  loginAction,
  LoginRequired,
  logoutAction,
} from "./Login";
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
