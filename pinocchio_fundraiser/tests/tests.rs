#[cfg(test)]
mod tests {

    use std::{io::Error, path::PathBuf};

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
        },
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };

    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::solana_program::program_pack::Pack;

    use crate::instructions::MakeData;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(crate::ID); 
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    const Days_IN_SECONDS: u64 = 86400;

    pub struct ReusableState {
    pub creator: Pubkey,
    pub alice: Pubkey,
    pub bob: Pubkey,
    pub carol: Pubkey,
    pub dave: Pubkey,
    pub creator_ata: Pubkey,
    pub alice_ata: Pubkey,
    pub bob_ata: Pubkey,
    pub carol_ata: Pubkey,
    pub dave_ata: Pubkey,
    pub fundraiser: Pubkey,
    pub fundraiser_bump: u8,
    pub vault: Pubkey,
    pub amount_to_raise: u64,
    pub duration: u64,
    pub mint_to_raise: Pubkey,
    pub ata_program: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    }

    fn program_id() -> Pubkey {
        PROGRAM_ID
    }

    fn setup() -> (LiteSVM, ReusableState) {

        let mut svm = LiteSVM::new();

        let mint_authority = Keypair::new();

        let creator = Keypair::new();

        let alice = Keypair::new();

        let bob = Keypair::new();

        let carol = Keypair::new();

        let dave = Keypair::new();

        svm.airdrop(&mint_authority.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&creator.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&alice.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&bob.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&carol.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        svm.airdrop(&dave.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let amount_to_raise: u64 = 100_000_000; // 100 tokens with 6 decimal places
        let duration: u64 = 2; // 2 days


        // Load program SO file
        msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));

        let bytes = include_bytes!("../../target/deploy/escrow.so");

        svm.add_program(program_id(), bytes);

        let mint_to_raise = CreateMint::new(&mut svm, &mint_authority)
            .decimals(6)
            .authority(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_to_raise);

        let creator_ata = CreateAssociatedTokenAccount::new(&mut svm, &mint_authority, &mint_to_raise)
            .owner(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("creator ATA : {}\n", creator_ata);

        let alice_ata = CreateAssociatedTokenAccount::new(&mut svm, &mint_authority, &mint_to_raise)
            .owner(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("Alice ATA : {}\n", alice_ata);

        let bob_ata = CreateAssociatedTokenAccount::new(&mut svm, &mint_authority, &mint_to_raise)
            .owner(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("Bob ATA : {}\n", bob_ata);

        let carol_ata = CreateAssociatedTokenAccount::new(&mut svm, &mint_authority, &mint_to_raise)
            .owner(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("Carol ATA : {}\n", carol_ata);

        let dave_ata = CreateAssociatedTokenAccount::new(&mut svm, &mint_authority, &mint_to_raise)
            .owner(&mint_authority.pubkey())
            .send()
            .unwrap();
        msg!("Dave ATA : {}\n", dave_ata);


        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let (fundraiser, fundraiser_bump) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), creator.pubkey().as_ref()],
            &PROGRAM_ID,
        );
        msg!("Fundraiser PDA: {}\n", fundraiser);

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser, // owner will be the escrow PDA
            &mint_to_raise,   // mint
        );
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        let reusable_state = ReusableState {
            creator,
            alice,
            bob,
            carol,
            dave,
            creator_ata,
            alice_ata,
            bob_ata,
            carol_ata,
            dave_ata,
            fundraiser,
            fundraiser_bump,
            vault,
            amount_to_raise,
            duration,
            mint_to_raise,
            ata_program: associated_token_program,
            token_program,
            system_program,
        };
        (svm, reusable_state)
    }

    pub fn process_create(svm: &mut LiteSVM, state: &ReusableState) -> Result<(), Error> {
        let mint_to_raise = state.mint_to_raise;
        let mint_authority = &state.maker;
        let maker_ata_a = state.maker_ata_a;
        let mint_b = state.mint_b;
        let vault = state.vault;
        let system_program = state.system_program;
        let token_program = state.token_program;
        let ata_program = state.ata_program;
        let escrow = state.escrow;

        MintTo::new(svm, &mint_authority, &mint_to_raise, &maker_ata_a, 1_000_000_000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100_000_000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 400_000_000; // 500 tokens with 6 decimal places

        let make_data_ix: MakeData = MakeData {
            make_amount: amount_to_give,
            take_amount: amount_to_receive,
        };

        let make_data_ser = make_data_ix.to_bytes();

        let make_data = [
            vec![crate::instructions::EscrowInstrctions::Make as u8], // Discriminator for "Make" instruction
            make_data_ser,
        ]
        .concat();

        let make_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(mint_authority.pubkey(), true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(ata_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&mint_authority.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&mint_authority], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();
        msg!("tx logs: {:#?}", tx.logs);
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        Ok(())
    }

    pub fn process_contribute(svm: &mut LiteSVM, state: &ReusableState) -> Result<(Pubkey, Pubkey), Error> {
        let mint_to_raise = state.mint_to_raise;
        let maker = &state.maker;
        let maker_ata_a = state.maker_ata_a;
        let maker_ata_b = state.maker_ata_b;
        let mint_b = state.mint_b;
        let vault = state.vault;
        let system_program = state.system_program;
        let token_program = state.token_program;
        let ata_program = state.ata_program;
        let escrow = state.escrow;

        let taker = Keypair::new();
        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let taker_ata_a = CreateAssociatedTokenAccount::new(svm, &taker, &mint_to_raise)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(svm, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA A: {}\nTaker ATA B {}", taker_ata_a, taker_ata_b);

        MintTo::new(svm, &maker, &mint_b, &taker_ata_b, 1_000_000_000)
            .send()
            .unwrap();

        let take_data = [
            vec![crate::instructions::EscrowInstrctions::Take as u8], // Discriminator for "Take" instruction
        ]
        .concat();

        let take_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(maker.pubkey(), false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(ata_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: take_data,
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();
        msg!("tx logs: {:#?}", tx.logs);
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        Ok((taker_ata_a, taker_ata_b))
    }

    pub fn process_checker(svm: &mut LiteSVM, state: &ReusableState) -> Result<(), Error> {
        let mint_to_raise = state.mint_to_raise;
        let maker = &state.maker;
        let maker_ata_a = state.maker_ata_a;
        let maker_ata_b = state.maker_ata_b;
        let mint_b = state.mint_b;
        let vault = state.vault;
        let system_program = state.system_program;
        let token_program = state.token_program;
        let ata_program = state.ata_program;
    }

    pub fn process_refund(svm: &mut LiteSVM, state: &ReusableState) -> Result<(), Error> {
        let mint_to_raise = state.mint_to_raise;
        let maker = &state.maker;
        let maker_ata_a = state.maker_ata_a;
        let maker_ata_b = state.maker_ata_b;
        let mint_b = state.mint_b;
        let vault = state.vault;
        let system_program = state.system_program;
        let token_program = state.token_program;
        let ata_program = state.ata_program;
    }

    #[test]
    pub fn test_create_instruction() {
        let (mut svm, state) = setup();

        let program_id = program_id();

        assert_eq!(program_id, PROGRAM_ID);
        make(&mut svm, &state).unwrap();

        let maker_ata_from_program = svm.get_account(&state.maker_ata_a).unwrap();

        let maker_deserialized_ata =
            spl_token::state::Account::unpack(maker_ata_from_program.data.as_slice()).unwrap();
        msg!("new user token_balance: {}", maker_deserialized_ata.amount);
    }

    #[test]
    pub fn test_contribute_instruction() {
        let (mut svm, state) = setup();

        let program_id = program_id();

        assert_eq!(program_id, PROGRAM_ID);
        make(&mut svm, &state).unwrap();
        let (taker_ata_a, taker_ata_b) = take(&mut svm, &state).unwrap();

        let taker_ata_a_from_program = svm.get_account(&taker_ata_a).unwrap();

        let taker_deserialized_ata_a =
            spl_token::state::Account::unpack(taker_ata_a_from_program.data.as_slice()).unwrap();
        msg!(
            "new user token_balance: {}",
            taker_deserialized_ata_a.amount
        );
        let taker_ata_b_from_program = svm.get_account(&taker_ata_b).unwrap();

        let taker_deserialized_ata_b =
            spl_token::state::Account::unpack(taker_ata_b_from_program.data.as_slice()).unwrap();
        msg!(
            "new user token_balance: {}",
            taker_deserialized_ata_b.amount
        );
    }

    #[test]
    pub fn test_checker_instruction() {}

    #[test]
    pub fn test_refund_instruction() {}
}