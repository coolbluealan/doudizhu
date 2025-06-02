import { createContext, useContext } from "react";

export const User = createContext("");
export default function useUser() {
  return useContext(User);
}
