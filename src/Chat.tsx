import { useState } from "react";

function ChatInput() {
  const [message, setMessage] = useState("");
  function send() {
    const msg = message.trim();
    if (msg == "") return;

    // socket stuff
  }

  return (
    <input
      type="text"
      value={message}
      onChange={(e) => {
        setMessage(e.target.value);
      }}
      onKeyDown={(e) => {
        if (e.key == "Enter") send();
      }}
    />
  );
}

type Message = {
  id: number;
  msg: string;
};
export default function Chat() {
  let messages: Array<Message> = [];
  return (
    <div className="chat">
      <div className="chat-history">
        {messages.map(({ id, msg }) => (
          <p key={id}>{msg}</p>
        ))}
      </div>
      <ChatInput />
    </div>
  );
}
