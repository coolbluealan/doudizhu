import "./chat.css";

import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router";

import fetchJson from "@/fetchJson";
import { ClientMsg, Msg, ServerMsg } from "@/types";

import useGame from "../GameContext";

type ChatProps = {
  initial: Msg[];
};
export default function Chat({ initial }: ChatProps) {
  const { lobbyCode } = useParams();
  const { players, idx, socket } = useGame();

  const [messages, setMessages] = useState(initial);
  const [loading, setLoading] = useState(true);
  const [hasMore, setHasMore] = useState(initial.length == 50);
  const [message, setMessage] = useState("");

  const chatRef = useRef<HTMLDivElement | null>(null);
  const sentinelRef = useRef<HTMLDivElement | null>(null);

  function scrollToBottom() {
    setTimeout(() => {
      const chat = chatRef.current;
      chat?.scrollTo({ top: chat.scrollHeight, behavior: "smooth" });
    });
  }

  // scroll on mount
  useEffect(() => {
    scrollToBottom();
    setLoading(false);
  }, []);

  // infinite scrollback
  useEffect(() => {
    if (!hasMore) return;

    async function loadMore() {
      if (loading) return;

      setLoading(true);
      const url = new URL(
        `/api/lobby/${lobbyCode}/chat`,
        window.location.origin,
      );
      if (messages.length) {
        url.searchParams.set("before", messages[0].time.toString());
      }

      const older = (await fetchJson(url.toString())) as Msg[];
      if (older.length) {
        setMessages((prev) => [...older, ...prev]);
      }

      setLoading(false);
      if (older.length < 50) {
        setHasMore(false);
      }
    }

    const sentinel = sentinelRef.current;
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        loadMore();
      }
    });
    if (sentinel) observer.observe(sentinel);

    return () => {
      if (sentinel) observer.unobserve(sentinel);
    };
  }, [lobbyCode, messages, loading, hasMore]);

  useEffect(() => {
    if (!socket) return;

    const handleChatMsg = (e: MessageEvent) => {
      const data: ServerMsg = JSON.parse(e.data);
      if ("Chat" in data) {
        setMessages((prev) => {
          // scroll to bottom if at bottom
          const chat = chatRef.current;
          if (
            chat &&
            chat.scrollHeight - chat.scrollTop - chat.clientHeight < 50
          ) {
            scrollToBottom();
          }
          return [...prev, data.Chat];
        });
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
    <div className="game-chat">
      <div ref={chatRef} className="messages overlay">
        <div ref={sentinelRef} className="sentinel">
          {loading && "Loading..."}
        </div>
        {messages.map(({ text, idx, time }) => (
          <div key={time}>
            <b className={`player-${idx}`}>
              {idx == 9 ? "Game" : players[idx].name}:{" "}
            </b>
            {text}
          </div>
        ))}
      </div>
      {idx != null && (
        <input
          className="chat-input"
          type="text"
          value={message}
          placeholder="Send a message"
          onChange={(e) => setMessage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key == "Enter") sendMessage();
          }}
        />
      )}
    </div>
  );
}
