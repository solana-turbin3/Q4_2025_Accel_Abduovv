import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  getAssociatedTokenAddressSync,
  getMintLen,
  createInitializeTransferHookInstruction,
  createInitializeMintInstruction,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  createTransferCheckedWithTransferHookInstruction,
} from "@solana/spl-token";
import {
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  SendTransactionError,
} from "@solana/web3.js";
import { WhitelistTransferHook } from "../target/types/whitelist_transfer_hook";

describe("whitelist-transfer-hook", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WhitelistTransferHook as Program<WhitelistTransferHook>;
  const admin = provider.wallet as anchor.Wallet;

  const user = Keypair.generate();
  const recipient = Keypair.generate();
  const mint2022 = Keypair.generate();

  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    admin.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint2022.publicKey,
    recipient.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Derive PDAs
  const [whitelist] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("whitelist"), user.publicKey.toBuffer()],
    program.programId
  );

  const [extraAccountMetaListPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("extra-account-metas"), mint2022.publicKey.toBuffer()],
    program.programId
  );

  it("✅ Initializes whitelist for a user", async () => {
    const tx = await program.methods
      .initializeWhitelist()
      .accounts({
        admin: admin.publicKey,
        whitelist,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("\nWhitelist initialized for user:", whitelist.toBase58());
    console.log("Tx:", tx);
  });

  it("✅ Toggles whitelist status", async () => {
    const tx = await program.methods
      .switchWhitelist()
      .accounts({
        admin: admin.publicKey,
        whitelist,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Whitelist switched for:", user.publicKey.toBase58());
    console.log("Tx:", tx);
  });

  it("✅ Creates Mint with Transfer Hook extension", async () => {
    const extensions = [ExtensionType.TransferHook];
    const mintLen = getMintLen(extensions);
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: mint2022.publicKey,
        space: mintLen,
        lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferHookInstruction(
        mint2022.publicKey,
        admin.publicKey,
        program.programId, // Transfer hook program
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint2022.publicKey,
        9,
        admin.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const sig = await sendAndConfirmTransaction(provider.connection, tx, [admin.payer, mint2022]);
    console.log("\nMint initialized with transfer hook:", sig);
  });

  it("✅ Creates token accounts and mints tokens", async () => {
    const amount = 100 * 10 ** 9;

    const tx = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        sourceTokenAccount,
        admin.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        admin.publicKey,
        destinationTokenAccount,
        recipient.publicKey,
        mint2022.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint2022.publicKey,
        sourceTokenAccount,
        admin.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );

    const sig = await sendAndConfirmTransaction(provider.connection, tx, [admin.payer]);
    console.log("Minted tokens Tx:", sig);
  });

  it("✅ Initializes ExtraAccountMetaList for transfer hook", async () => {
    const ix = await program.methods
      .initializeTransferHook()
      .accounts({
        payer: admin.publicKey,
        mint: mint2022.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const tx = new Transaction().add(ix);
    const sig = await sendAndConfirmTransaction(provider.connection, tx, [admin.payer]);
    console.log("\nExtraAccountMetaList created:", extraAccountMetaListPDA.toBase58());
    console.log("Tx:", sig);
  });

  it("✅ Executes transfer (hook validation)", async () => {
    const amount = 1 * 10 ** 9;
    const amountBigInt = BigInt(amount);

    const ix = await createTransferCheckedWithTransferHookInstruction(
      provider.connection,
      sourceTokenAccount,
      mint2022.publicKey,
      destinationTokenAccount,
      admin.publicKey,
      amountBigInt,
      9,
      [],
      "confirmed",
      TOKEN_2022_PROGRAM_ID
    );

    const tx = new Transaction().add(ix);

    try {
      const sig = await sendAndConfirmTransaction(provider.connection, tx, [admin.payer]);
      console.log("\nTransfer executed successfully:", sig);
    } catch (error) {
      if (error instanceof SendTransactionError) {
        console.error("Transfer failed:", error.logs);
      } else {
        console.error("Unexpected error:", error);
      }
    }
  });
});
