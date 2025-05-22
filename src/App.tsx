import { createBrowserRouter, RouterProvider } from "react-router";
import Home from "./Home";
import { Login, loginAction, LoginRequired } from "./Login";
import Game from "./Game";

const router = createBrowserRouter([
  {
    path: "/login",
    action: loginAction,
    element: <Login />,
  },
  {
    element: <LoginRequired />,
    children: [
      { path: "/", element: <Home /> },
      { path: "/lobby/:lobby_code", element: <Game /> },
    ],
  },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
