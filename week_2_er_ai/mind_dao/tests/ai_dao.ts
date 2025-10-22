import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { MindDao } from "../target/types/mind_dao";

describe("miiiiiiiiiiiiinnnnnnnnnnnddddddddd dao", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.aiDao as Program<MindDao>;

    it("Initialize the agent", async () => {
      
    });
    it("Create a rejected proposal", async () => {
      
    });
    it("Create a accepted proposal", async () => {
      
    });
    it("Vote on a proposal", async () => {
      
    });
  });

