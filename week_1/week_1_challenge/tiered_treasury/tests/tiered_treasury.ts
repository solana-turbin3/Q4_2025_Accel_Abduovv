import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TieredTreasury } from "../target/types/tiered_treasury";

describe("tiered_treasury", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.tieredTreasury as Program<TieredTreasury>;

  it("Say Hello!", async () => {
    const tx = await program.methods.sayHello().rpc();
    console.log("Your transaction signature", tx);
  });
});
