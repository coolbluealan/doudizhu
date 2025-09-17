import "./hand.css";

import { useState } from "react";
import { Form, useParams } from "react-router";

import FormError from "@/FormError";
import useUser from "@/login/UserContext";
import { ClientMsg } from "@/types";

import Card from "../card/Card";
import useGame from "../GameContext";

type HandProps = {
  hand: number[];
};
export default function Hand({ hand }: HandProps) {
  const [selected, setSelected] = useState<boolean[]>(
    new Array(hand.length).fill(false),
  );

  function toggle(i: number) {
    setSelected((prev) => {
      const next = [...prev];
      next[i] = !next[i];
      return next;
    });
  }

  return (
    <>
      <div className="game-hand cards">
        {hand.map((card, i) => (
          <Card
            key={i}
            card={card}
            small={hand.length > 20}
            selected={selected[i]}
            onMouseDown={() => toggle(i)}
            onMouseEnter={(e) => {
              if (e.buttons & 1) toggle(i);
            }}
          />
        ))}
      </div>
      <Actions
        hand={hand}
        selected={selected}
        clearHand={() => setSelected(new Array(hand.length).fill(false))}
      />
    </>
  );
}

type ActionsProps = {
  hand: number[];
  selected: boolean[];
  clearHand: () => void;
};
function Actions({ hand, selected, clearHand }: ActionsProps) {
  const { lobbyCode } = useParams();

  const user = useUser();
  const { status, players, idx, game, socket } = useGame();

  if (idx == undefined) {
    if (status == "Lobby" && players.length < 4) {
      return (
        <div className="game-actions">
          <Form action="/" method="POST">
            <button className="btn-primary" type="submit">
              Join game as {user}
            </button>
            <input type="hidden" name="lobbyCode" value={lobbyCode} />
            <FormError />
          </Form>
        </div>
      );
    }
    return null;
  }

  const notTurn = idx != game?.turn;

  function startBtn(text: string) {
    return (
      <button
        className="btn-primary"
        onClick={() =>
          socket?.send(JSON.stringify({ Start: null } satisfies ClientMsg))
        }
      >
        {text}
      </button>
    );
  }

  let actions;
  if (status == "Lobby") {
    if (players.length >= 3) {
      actions = startBtn("Start Game");
    }
  } else if (status == "Bidding") {
    function bid(val: number) {
      return () =>
        socket?.send(JSON.stringify({ Bid: val } satisfies ClientMsg));
    }

    actions = (
      <>
        <button
          className="btn-primary"
          onClick={bid(1)}
          disabled={notTurn || game!.bid >= 1}
        >
          1
        </button>
        <button
          className="btn-primary"
          onClick={bid(2)}
          disabled={notTurn || game!.bid >= 2}
        >
          2
        </button>
        <button className="btn-primary" onClick={bid(3)} disabled={notTurn}>
          3
        </button>
        <button className="btn-secondary" onClick={bid(0)} disabled={notTurn}>
          Pass
        </button>
      </>
    );
  } else if (status == "Playing") {
    function play(cards: number[]) {
      socket?.send(JSON.stringify({ Play: cards } satisfies ClientMsg));
    }
    actions = (
      <>
        <button
          className="btn-primary"
          onClick={() => {
            play(hand.filter((_, i) => selected[i]));
            clearHand();
          }}
          disabled={notTurn || !selected.includes(true)}
        >
          Play
        </button>
        <button
          className="btn-secondary"
          onClick={() => play([])}
          disabled={notTurn}
        >
          Pass
        </button>
      </>
    );
  } else {
    actions = startBtn("Play Again");
  }

  return (
    <div className="game-actions">
      {status == "Playing" && (
        <button onClick={clearHand} disabled={!selected.includes(true)}>
          Clear
        </button>
      )}
      {actions}
    </div>
  );
}
