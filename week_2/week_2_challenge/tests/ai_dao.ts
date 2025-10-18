import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AiDao } from "../target/types/ai_dao";

describe("ai_dao", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.aiDao as Program<AiDao>;

  it("Say Hello!", async () => {
    const tx = await program.methods.sayHello().rpc();
    console.log("Your transaction signature", tx);
  });
});
