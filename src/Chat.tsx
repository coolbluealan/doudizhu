import { useContext, useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { fetchJson } from "./Error";
import { GameContext } from "./Game";
import { ClientMsg, Msg, ServerMsg } from "./types";

type ChatProps = {
  initial: Msg[];
};

export default function Chat({ initial }: ChatProps) {
  const { lobbyCode } = useParams();
  const { players, idx, socket } = useContext(GameContext);

  const [messages, setMessages] = useState(initial);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(initial.length == 50);
  const [message, setMessage] = useState("");

  const sentinel = useRef<HTMLDivElement | null>(null);

  async function loadMore() {
    if (loading || !hasMore) return;

    setLoading(true);
    const url = new URL(`/api/lobby/${lobbyCode}/chat`, window.location.origin);
    if (messages.length) {
      url.searchParams.set("before", messages[0].time.toString());
    }

    const older = (await fetchJson(url.toString())) as Msg[];
    if (older.length) {
      setMessages((prev) => [...older, ...prev]);
    } else {
      setHasMore(false);
    }
    setLoading(false);
  }

  // infinite scrollback
  useEffect(() => {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        loadMore();
      }
    });
    if (sentinel.current) observer.observe(sentinel.current);

    return () => {
      if (sentinel.current) observer.unobserve(sentinel.current);
    };
  }, []);

  useEffect(() => {
    if (!socket) return;

    const handleChatMsg = (e: MessageEvent) => {
      const data: ServerMsg = JSON.parse(e.data);
      if ("Chat" in data) {
        setMessages((prev) => [...prev, data.Chat]);
      }
    };
    socket.addEventListener("message", handleChatMsg);

    return () => {
      socket.removeEventListener("message", handleChatMsg);
    };
  }, [socket]);

  function sendMessage() {
    const msg = message.trim();
    if (msg && socket) {
      socket.send(
        JSON.stringify({
          Chat: msg,
        } satisfies ClientMsg),
      );
      setMessage("");
    }
  }

  return (
    <div>
      <ul>
        <div ref={sentinel}>{loading && "Loading..."}</div>
        {messages.map(({ text, idx, time }) => (
          <li key={time}>
            <b className={`msg-${idx}`}>
              {idx == 3 ? "Game" : players[idx].name}:{" "}
            </b>
            {text}
          </li>
        ))}
      </ul>
      {idx != null && (
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key == "Enter") sendMessage();
          }}
        />
      )}
    </div>
  );
}
