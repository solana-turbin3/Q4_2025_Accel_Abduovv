#[cfg(test)]mod tests {

    use std::io::Error;

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::{clock::Clock, msg, rent::Rent, sysvar::SysvarId},
        },
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };

    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk::{sysvar::clock};
    use solana_sdk_ids::system_program;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account, solana_program::program_pack::Pack};
    use spl_token::state::Account as TokenAccount;
    use solana_program::{clock::Clock as SolClock, sysvar::Sysvar};

    use crate::{instructions::{ContributeData, CreateData, ProgramInstruction}, states::{Fundraiser, Contribute}};

    const PROGRAM_ID_STR: &str = "AYQEqZMiyxTfz9m9fcoQuPu3SA6wtbD2XCjEMMgfiXJH";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = spl_associated_token_account::ID;
    const SYSTEM_PROGRAM_ID: Pubkey = system_program::ID;
    const DAYS_IN_SECONDS: i64 = 86400;
    const SLOTS_PER_DAY: u64 = (DAYS_IN_SECONDS as f64 / 0.4) as u64;
    const DECIMALS: u8 = 6;
    const TOKENS_TO_MINT: u64 = 1_000_000u64;  // 1M UI tokens per user (raw: * 10^6)

    pub struct ReusableState {
        pub creator: Keypair,
        pub alice: Keypair,
        pub bob: Keypair,
        pub carol: Keypair,
        pub dave: Keypair,
        pub creator_ata: Pubkey,
        pub alice_ata: Pubkey,
        pub bob_ata: Pubkey,
        pub carol_ata: Pubkey,
        pub dave_ata: Pubkey,
        pub fundraiser: Pubkey,
        pub fundraiser_bump: u8,
        pub alice_contribute: Pubkey,
        pub bob_contribute: Pubkey,
        pub vault: Pubkey,
        pub mint_to_raise: Pubkey,
        pub ata_program: Pubkey,
        pub token_program: Pubkey,
        pub system_program: Pubkey,
        pub mint_authority: Keypair,
    }

    fn program_id() -> Pubkey {
        PROGRAM_ID_STR.parse().unwrap()
    }

    fn setup() -> (LiteSVM, ReusableState) {
        let mut svm = LiteSVM::new();

        let mint_authority = Keypair::new();
        let creator = Keypair::new();
        let alice = Keypair::new();
        let bob = Keypair::new();
        let carol = Keypair::new();
        let dave = Keypair::new();
        let ata_program = ASSOCIATED_TOKEN_PROGRAM_ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        svm.airdrop(&mint_authority.pubkey(), 10 * LAMPORTS_PER_SOL as u64).unwrap();
        svm.airdrop(&creator.pubkey(), 100 * LAMPORTS_PER_SOL as u64).unwrap();
        svm.airdrop(&alice.pubkey(), 10 * LAMPORTS_PER_SOL as u64).unwrap();
        svm.airdrop(&bob.pubkey(), 10 * LAMPORTS_PER_SOL as u64).unwrap();
        svm.airdrop(&carol.pubkey(), 10 * LAMPORTS_PER_SOL as u64).unwrap();
        svm.airdrop(&dave.pubkey(), 10 * LAMPORTS_PER_SOL as u64).unwrap();

        let so_bytes = include_bytes!("/home/abduo/Q4_2025_Accel_Abduovv/pinocchio_fundraiser/target/deploy/pinocchio_fundraiser.so");
        svm.add_program(program_id(), &so_bytes.to_vec());

        let mint_to_raise = CreateMint::new(&mut svm, &mint_authority)
            .decimals(DECIMALS)
            .authority(&mint_authority.pubkey())
            .send()
            .unwrap();

        // Creator ATA + mint
        let creator_ata = get_associated_token_address(&creator.pubkey(), &mint_to_raise);
        let ix = create_associated_token_account(
            &creator.pubkey(),
            &creator.pubkey(),
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&creator.pubkey()),
            &[&creator],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        MintTo::new(
            &mut svm,
            &mint_authority,
            &mint_to_raise,
            &creator_ata,
            TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32),
        ).send().unwrap();

        // Alice ATA + mint
        let alice_ata = get_associated_token_address(&alice.pubkey(), &mint_to_raise);
        let ix = create_associated_token_account(
            &alice.pubkey(),
            &alice.pubkey(),
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&alice.pubkey()),
            &[&alice],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        MintTo::new(
            &mut svm,
            &mint_authority,
            &mint_to_raise,
            &alice_ata,
            TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32),
        ).send().unwrap();

        // Bob ATA + mint
        let bob_ata = get_associated_token_address(&bob.pubkey(), &mint_to_raise);
        let ix = create_associated_token_account(
            &bob.pubkey(),
            &bob.pubkey(),
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&bob.pubkey()),
            &[&bob],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        MintTo::new(
            &mut svm,
            &mint_authority,
            &mint_to_raise,
            &bob_ata,
            TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32),
        ).send().unwrap();

        // Carol ATA + mint (for completeness)
        let carol_ata = get_associated_token_address(&carol.pubkey(), &mint_to_raise);
        let ix = create_associated_token_account(
            &carol.pubkey(),
            &carol.pubkey(),
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&carol.pubkey()),
            &[&carol],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        MintTo::new(
            &mut svm,
            &mint_authority,
            &mint_to_raise,
            &carol_ata,
            TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32),
        ).send().unwrap();

        // Dave ATA + mint (for completeness)
        let dave_ata = get_associated_token_address(&dave.pubkey(), &mint_to_raise);
        let ix = create_associated_token_account(
            &dave.pubkey(),
            &dave.pubkey(),
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&dave.pubkey()),
            &[&dave],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();
        MintTo::new(
            &mut svm,
            &mint_authority,
            &mint_to_raise,
            &dave_ata,
            TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32),
        ).send().unwrap();

        let (fundraiser, fundraiser_bump) = Pubkey::find_program_address(
            &[b"fundraiser", creator.pubkey().as_ref()],
            &program_id(),
        );

        let vault = get_associated_token_address(&fundraiser, &mint_to_raise);
        let ix = create_associated_token_account(
            &creator.pubkey(),
            &fundraiser,
            &mint_to_raise,
            &token_program,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&creator.pubkey()),
            &[&creator],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).unwrap();

        let alice_contribute = Pubkey::find_program_address(
            &[b"contribute", alice.pubkey().as_ref(), fundraiser.as_ref()],
            &program_id(),
        ).0;

        let bob_contribute = Pubkey::find_program_address(
            &[b"contribute", bob.pubkey().as_ref(), fundraiser.as_ref()],
            &program_id(),
        ).0;

        let state = ReusableState {
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
            alice_contribute,
            bob_contribute,
            vault,
            mint_to_raise,
            ata_program,
            token_program,
            system_program,
            mint_authority,
        };

        (svm, state)
    }

    pub fn warp_time_days(svm: &mut LiteSVM, days: u64) {
        let mut clock = SolClock::get().expect("Failed to get clock");
        clock.unix_timestamp += (days as i64) * DAYS_IN_SECONDS;
        svm.set_sysvar(&clock);
    }

    pub fn process_create(svm: &mut LiteSVM, state: &ReusableState) {
        let creator = &state.creator;
        let create_data = CreateData {
            amount_to_raise: 3_000 * 10u64.pow(DECIMALS as u32),  // 3k UI target to match small contributes
            duration: 7,
            _padding: [0; 7],
        };
        let full_data = [ProgramInstruction::Create as u8]
            .iter()
            .chain(create_data.to_bytes().iter())
            .cloned()
            .collect::<Vec<_>>();

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new_readonly(state.mint_to_raise, false),
                AccountMeta::new(state.fundraiser, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new_readonly(state.system_program, false),
                AccountMeta::new_readonly(state.token_program, false),
                AccountMeta::new_readonly(state.ata_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: full_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&creator.pubkey()),
            &[creator],
            svm.latest_blockhash(),
        );

        let result = svm.send_transaction(tx).expect("Create transaction failed");
        msg!("Create logs: {:?}", result.logs);
    }

    pub fn process_contribute(
        svm: &mut LiteSVM,
        state: &ReusableState,
        contributor: &Keypair,
        contributor_ata: Pubkey,
        contribute_pda: Pubkey,
        ui_amount: u64,  // UI tokens
    ) {
        let raw_amount = ui_amount * 10u64.pow(DECIMALS as u32);
        let contribute_data = ContributeData {
            amount_contributed: raw_amount,
        };
        let full_data = [ProgramInstruction::Contribute as u8]
            .iter()
            .chain(contribute_data.to_bytes().iter())
            .cloned()
            .collect::<Vec<_>>();

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(state.creator.pubkey(), false),
                AccountMeta::new(contribute_pda, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new_readonly(state.mint_to_raise, false),
                AccountMeta::new(state.fundraiser, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new_readonly(state.system_program, false),
                AccountMeta::new_readonly(state.token_program, false),
                AccountMeta::new_readonly(state.ata_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: full_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&contributor.pubkey()),
            &[contributor],
            svm.latest_blockhash(),
        );

        let result = svm.send_transaction(tx).expect("Contribute transaction failed");
        msg!("Contribute logs: {:?}", result.logs);
    }

    pub fn process_checker(svm: &mut LiteSVM, state: &ReusableState) {
        let creator = &state.creator;
        let full_data = vec![ProgramInstruction::Checker as u8];

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(creator.pubkey(), true),
                AccountMeta::new(state.creator_ata, false),
                AccountMeta::new(state.fundraiser, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new_readonly(state.system_program, false),
                AccountMeta::new_readonly(state.token_program, false),
                AccountMeta::new_readonly(state.ata_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: full_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&creator.pubkey()),
            &[creator],
            svm.latest_blockhash(),
        );

        let tx_result = svm.send_transaction(tx).expect("Checker transaction failed");
        msg!("Checker logs: {:#?}", tx_result.logs);
        msg!("\n\nChecker transaction successful");
        msg!("CUs Consumed: {}", tx_result.compute_units_consumed);
    }

    pub fn process_refund(
        svm: &mut LiteSVM,
        state: &ReusableState,
        contributor: &Keypair,
        contributor_ata: Pubkey,
        contribute_pda: Pubkey,
    ) {
        let full_data = vec![ProgramInstruction::Refund as u8];

        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true),
                AccountMeta::new_readonly(state.creator.pubkey(), false),
                AccountMeta::new(contribute_pda, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new_readonly(state.mint_to_raise, false),
                AccountMeta::new_readonly(state.fundraiser, false),
                AccountMeta::new(state.vault, false),
                AccountMeta::new_readonly(state.system_program, false),
                AccountMeta::new_readonly(state.token_program, false),
                AccountMeta::new_readonly(state.ata_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: full_data,
        };

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&contributor.pubkey()),
            &[contributor],
            svm.latest_blockhash(),
        );

        let tx_result = svm.send_transaction(tx).expect("Refund transaction failed");
        msg!("Refund logs: {:#?}", tx_result.logs);
        msg!("\n\nRefund transaction successful");
        msg!("CUs Consumed: {}", tx_result.compute_units_consumed);
    }

    #[test]
    fn test_create_instruction() {
        let (mut svm, state) = setup();
        process_create(&mut svm, &state);

        let vault_account = svm.get_account(&state.vault).unwrap();
        let vault_token: TokenAccount = TokenAccount::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_token.amount, 0);
    }

    #[test]
    fn test_contribute_instruction() {
        let (mut svm, state) = setup();
        process_create(&mut svm, &state);
        process_contribute(&mut svm, &state, &state.alice, state.alice_ata, state.alice_contribute, 250).unwrap();  // 250 UI

        let vault_account = svm.get_account(&state.vault).unwrap();
        let vault_token: TokenAccount = TokenAccount::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_token.amount, 250 * 10u64.pow(DECIMALS as u32));
    }

    #[test]
    fn test_checker_instruction() {
        let (mut svm, state) = setup();
        process_create(&mut svm, &state);
        process_contribute(&mut svm, &state, &state.alice, state.alice_ata, state.alice_contribute, 1_000).unwrap();  // 1k UI
        process_contribute(&mut svm, &state, &state.bob, state.bob_ata, state.bob_contribute, 2_000).unwrap();  // 2k UI

        let vault_account = svm.get_account(&state.vault).unwrap();
        let vault_token: TokenAccount = TokenAccount::unpack(&vault_account.data).unwrap();
        assert!(vault_token.amount >= 3_000 * 10u64.pow(DECIMALS as u32));

        process_checker(&mut svm, &state);

        let creator_ata_account = svm.get_account(&state.creator_ata).unwrap();
        let creator_token: TokenAccount = TokenAccount::unpack(&creator_ata_account.data).unwrap();
        assert!(creator_token.amount >= 3_000 * 10u64.pow(DECIMALS as u32) + TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32));  // Original + raised
        assert!(svm.get_account(&state.vault).is_none());
        assert!(svm.get_account(&state.fundraiser).is_none());
    }

    #[test]
    fn test_refund_instruction() {
        let (mut svm, state) = setup();
        process_create(&mut svm, &state);
        process_contribute(&mut svm, &state, &state.alice, state.alice_ata, state.alice_contribute, 250).unwrap();

        warp_time_days(&mut svm, 2);

        process_refund(&mut svm, &state, &state.alice, state.alice_ata, state.alice_contribute);

        let alice_ata_account = svm.get_account(&state.alice_ata).unwrap();
        let alice_token: TokenAccount = TokenAccount::unpack(&alice_ata_account.data).unwrap();
        assert_eq!(alice_token.amount, TOKENS_TO_MINT * 10u64.pow(DECIMALS as u32));  // Back to original
        assert!(svm.get_account(&state.alice_contribute).is_none());
    }
}