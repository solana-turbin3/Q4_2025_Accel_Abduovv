import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { GetCommitmentSignature } from "@magicblock-labs/ephemeral-rollups-sdk";
import { ErStateAccount } from "../target/types/er_state_account";

describe("🧩 er-state-account", () => {
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

  console.log("Base Layer RPC:", provider.connection.rpcEndpoint);
  console.log("Ephemeral Rollup RPC:", providerEphemeralRollup.connection.rpcEndpoint);
  console.log("Current Wallet:", user.publicKey.toBase58());

  const program = anchor.workspace.erStateAccount as Program<ErStateAccount>;

  const [userAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("user"), user.publicKey.toBuffer()],
    program.programId
  );

  async function logUserAccount() {
    try {
      const account = await program.account.userAccount.fetch(userAccount);
      console.log("Random Number:   ", account.randomNumber?.toString() ?? "N/A");
    } catch (err) {
      console.error("Failed to fetch UserAccount:", err);
    }
  }

  before(async () => {
    const balance = await provider.connection.getBalance(user.publicKey);
    console.log("\nCurrent balance:", balance / LAMPORTS_PER_SOL, "SOL\n");
  });

  it("Initialize User Account", async () => {
    const tx = await program.methods.initialize().accountsPartial({
      user: user.publicKey,
      userAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    console.log("User Account Initialized:", tx);
    await logUserAccount();
  });

  it("Update State", async () => {
    const tx = await program.methods.update().accountsPartial({
      user: user.publicKey,
      userAccount,
    }).rpc();

    console.log("User Account Updated:", tx);
    await logUserAccount();
  });

  it("Delegate to Ephemeral Rollup", async () => {
    const tx = await program.methods.delegate().accountsPartial({
      user: user.publicKey,
      userAccount,
      validator: new PublicKey("MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57"),
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc({ skipPreflight: true });

    console.log("Delegated to Rollup:", tx);
  });

  it("Update After Delegation", async () => {
    const tx = await program.methods.update().accountsPartial({
      user: user.publicKey,
      userAccount,
    }).rpc();

    console.log("Updated After Delegation:", tx);
    await logUserAccount();
  });

  it("Update and commit to Base Layer", async () => {
    let tx = await program.methods.updateCommit().accountsPartial({
      user: providerEphemeralRollup.wallet.publicKey,
      userAccount,
    }).transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;
    tx.recentBlockhash = (await providerEphemeralRollup.connection.getLatestBlockhash()).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);

    const txHash = await providerEphemeralRollup.sendAndConfirm(tx, [], { skipPreflight: false });
    await GetCommitmentSignature(txHash, providerEphemeralRollup.connection);

    console.log("Committed to Base Layer:", txHash);
    await logUserAccount();
  });

  it("Undelegate from Rollup", async () => {
    console.log("User Account:", userAccount.toBase58());

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
      userAccount,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc();

    console.log("User Account Closed:", tx);
  });
});
