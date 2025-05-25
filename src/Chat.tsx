import { useContext, useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { fetchJson } from "./Error";
import { GameContext } from "./Game";
import { Msg } from "./types";

export default function Chat() {
  const { lobbyCode } = useParams();
  const { players, idx, messages, setMessages, sendMessage } =
    useContext(GameContext);

  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(messages.length == 50);
  const [message, setMessage] = useState("");

  const sentinel = useRef<HTMLDivElement | null>(null);

  async function loadMore() {
    if (loading || !hasMore) {
      return;
    }

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

  useEffect(() => {
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        loadMore();
      }
    });

    if (sentinel.current) {
      observer.observe(sentinel.current);
    }
    return () => {
      if (sentinel.current) {
        observer.unobserve(sentinel.current);
      }
    };
  }, [messages]);

  return (
    <div>
      <ul>
        <div ref={sentinel} />
        {loading && <div>Loading...</div>}
        {messages.map(({ text, idx, time }) => (
          <li key={time}>
            {idx == 3 ? "Game" : players[idx].name}: {text}
          </li>
        ))}
      </ul>
      {idx != null && (
        <div>
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => {
              if (e.key == "Enter") if (sendMessage(message)) setMessage("");
            }}
          />
        </div>
      )}
    </div>
  );
}
