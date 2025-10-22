import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey} from "@solana/web3.js";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { PlayerPowers } from "../target/types/player_powers";

describe("Player Powers, with Ephemeral Rollup and VRF", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const user = anchor.Wallet.local();

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT || "https://devnet.magicblock.app/",
      { wsEndpoint: process.env.EPHEMERAL_WS_ENDPOINT || "wss://devnet.magicblock.app/" }
    ),
    anchor.Wallet.local()
  );
  const dna = Math.floor(Math.random() * 255);
  console.log("Base Layer RPC:", provider.connection.rpcEndpoint);
  console.log("Ephemeral Rollup RPC:", providerEphemeralRollup.connection.rpcEndpoint);
  console.log("Current Wallet:", user.publicKey.toBase58());
  console.log("Random DNA:", dna);


  const program = anchor.workspace.erStateAccount as Program<PlayerPowers>;

  const [PlayerPowers] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), user.publicKey.toBuffer(), Buffer.from(new Uint8Array([dna]))],
    program.programId
  );

  async function logPlayer() {
    try {
      const account = await program.account.player.fetch(PlayerPowers);
      console.log("Class:   ", account.class?.toString() ?? "N/A");
      console.log("Attack:   ", account.attack?.toString() ?? "N/A");
      console.log("Defense:   ", account.defense?.toString() ?? "N/A");
      console.log("Stamina:   ", account.stamina?.toString() ?? "N/A");
    } catch (err) {
      console.error("Failed to fetch Player:", err);
    }
  }

  before(async () => {
    const balance = await provider.connection.getBalance(user.publicKey);
    console.log("\nCurrent balance:", balance / LAMPORTS_PER_SOL, "SOL\n");
    const sig = await provider.connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig, "confirmed") ;
    console.log("\nAfter airdrop:", balance / LAMPORTS_PER_SOL, "SOL\n");
  });

  it("Initialize User Account", async () => {
    
    const tx = await program.methods.initialize(dna).accountsPartial({
      user: user.publicKey,
      player: PlayerPowers,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    console.log("User Account Initialized:", tx);
    await logPlayer();
  });

  it("Update State", async () => {
    const tx = await program.methods.update().accountsPartial({
      user: user.publicKey,
      player: PlayerPowers,
    }).rpc();

    console.log("User Account Updated:", tx);
    await logPlayer();
  });

  it("Delegate to Ephemeral Rollup", async () => {
    const tx = await program.methods.delegate().accountsPartial({
      user: user.publicKey,
      player: PlayerPowers,
      validator: new PublicKey("mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev"),
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc({ skipPreflight: true });

    console.log("Delegated to Rollup:", tx);
  });

  it("Update and commit to Base Layer", async () => {
    let tx = await program.methods.updateCommit().accountsPartial({
      user: providerEphemeralRollup.wallet.publicKey,
      player: PlayerPowers,
    }).transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], { skipPreflight: false });
    await GetCommitmentSignature(txHash, providerEphemeralRollup.connection);

    console.log("Committed to Base Layer:", txHash);
    await logPlayer();
  });

  it("Undelegate from Rollup", async () => {
    console.log("Player Account:", PlayerPowers.toBase58());

    let tx = await program.methods.undelegate().accounts({
      user: providerEphemeralRollup.wallet.publicKey,
    }).transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], { skipPreflight: false });
    await GetCommitmentSignature(txHash, providerEphemeralRollup.connection);

    console.log("Undelegated from Rollup:", txHash);
  });

  it("Close User Account", async () => {
    const tx = await program.methods.close().accountsPartial({
      user: user.publicKey,
      player: PlayerPowers,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    console.log("Player Account Closed:", tx);
  });
});
