const rankMap = [
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "10",
  "J",
  "Q",
  "K",
  "A",
  "2",
  "J",
];
export function rank(card: number) {
  return rankMap[Math.floor(card / 4)];
}

const suitMap = ["clubs", "diamonds", "hearts", "spades"];
export function suit(card: number) {
  if (card >= 52) return "joker";
  return suitMap[card % 4];
}

const emojiSuitMap = ["♣️", "♦️", "♥️", "♠️"];
function emojiSuit(card: number) {
  if (card == 52) return "🃟";
  if (card == 53) return "🃏";
  return emojiSuitMap[card % 4];
}
export function display(cards: number[]) {
  return cards.map((card) => rank(card) + emojiSuit(card)).join(", ");
}

const colorMap = ["black", "red", "red", "black"];
export function color(card: number) {
  return colorMap[card % 4];
}
