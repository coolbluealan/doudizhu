import ClubsSvg from "@/assets/clubs.svg?react";
import DiamondsSvg from "@/assets/diamonds.svg?react";
import HeartsSvg from "@/assets/hearts.svg?react";
import SpadesSvg from "@/assets/spades.svg?react";

export default function SuitDefs() {
  return (
    <svg style={{ display: "none" }}>
      <defs>
        <symbol id="clubs">
          <ClubsSvg />
        </symbol>
        <symbol id="diamonds">
          <DiamondsSvg />
        </symbol>
        <symbol id="hearts">
          <HeartsSvg />
        </symbol>
        <symbol id="spades">
          <SpadesSvg />
        </symbol>
      </defs>
    </svg>
  );
}
