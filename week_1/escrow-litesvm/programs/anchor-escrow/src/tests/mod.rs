#[cfg(test)]
mod tests {
    use {
        anchor_lang::{
            prelude::{msg, Clock},
            solana_program::{program_pack::Pack},
            InstructionData,
            ToAccountMetas,
        }, anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        }, litesvm::LiteSVM, litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID,
            CreateAssociatedTokenAccount,
            CreateMint,
            MintTo,
        }, solana_instruction::Instruction, solana_keypair::Keypair, solana_message::Message, solana_native_token::LAMPORTS_PER_SOL, solana_pubkey::Pubkey, solana_sdk_ids::{system_program::ID as SYSTEM_PROGRAM_ID, sysvar::clock::ID as Clock_ID}, solana_signer::Signer, solana_transaction::Transaction, std::path::PathBuf,
    };

    static PROGRAM_ID: Pubkey = crate::ID;
    const OPEN_IN: i64 = 1;

    fn setup() -> (
        LiteSVM,
        Keypair,
        Keypair,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        let mut program = LiteSVM::new();
        let maker = Keypair::new();
        let taker = Keypair::new();
        let payer = Keypair::new();

        program.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        program.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        program.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/anchor_escrow.so");
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
        program.add_program(PROGRAM_ID, &program_data);

        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();

        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();

        let associated_token_program = spl_associated_token_account::ID;
        let token_program: Pubkey = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        (
            program,
            maker,
            taker,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        )
    }

    fn make_flow(
        program: &mut LiteSVM,
        payer: &Keypair,
        maker: &Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_a: Pubkey,
        _maker_ata_b: Pubkey,
        associated_token_program: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
    ) -> (Pubkey, Pubkey) {
        let maker_pk = maker.pubkey();
        let escrow =
            Pubkey::find_program_address(&[b"escrow", maker_pk.as_ref(), &123u64.to_le_bytes()], &PROGRAM_ID).0;
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

        MintTo::new(program, payer, &mint_a, &maker_ata_a, 1_000_000_000)
            .send()
            .unwrap();

        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker_pk,
                mint_a,
                mint_b,
                maker_ata_a,
                escrow,
                vault,
                associated_token_program,
                token_program,
                system_program,
                clock: Clock_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
                open_in: OPEN_IN,
            }
            .data(),
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let blockhash = program.latest_blockhash();
        let tx = Transaction::new(&[maker], message, blockhash);
        let result = program.send_transaction(tx).unwrap();

        msg!("Make successful | CU: {} | Sig: {}", result.compute_units_consumed, result.signature);

        (escrow, vault)
    }

    fn take_flow(
        program: &mut LiteSVM,
        payer: &Keypair,
        maker: &Keypair,
        taker: &Keypair,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_b: Pubkey,
        taker_ata_a: Pubkey,
        taker_ata_b: Pubkey,
        escrow: Pubkey,
        vault: Pubkey,
        associated_token_program: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
    ) {
        MintTo::new(program, payer, &mint_b, &taker_ata_b, 1_000_000_000)
            .send()
            .unwrap();

        let mut new_clock = program.get_sysvar::<Clock>();
        new_clock.unix_timestamp = OPEN_IN + 2;
        program.set_sysvar::<Clock>(&new_clock);

        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: taker.pubkey(),
                maker: maker.pubkey(),
                mint_a,
                mint_b,
                taker_ata_a,
                taker_ata_b,
                maker_ata_b,
                escrow,
                vault,
                clock: Clock_ID,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let blockhash = program.latest_blockhash();
        let tx = Transaction::new(&[taker], message, blockhash);
        let result = program.send_transaction(tx).unwrap();

        msg!("Take successful | CU: {} | Sig: {}", result.compute_units_consumed, result.signature);
    }

    #[test]
    fn test_make() {
        let (
            mut program,
            maker,
            _taker,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            _taker_ata_a,
            _taker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let (escrow, vault) = make_flow(
            &mut program,
            &payer,
            &maker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        );

        let vault_acc = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_acc.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);
    }

    #[test]
    fn test_take() {
        let (
            mut program,
            maker,
            taker,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let (escrow, vault) = make_flow(
            &mut program,
            &payer,
            &maker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        );

        take_flow(
            &mut program,
            &payer,
            &maker,
            &taker,
            mint_a,
            mint_b,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
            associated_token_program,
            token_program,
            system_program,
        );

        let taker_ata_b_acc = program.get_account(&taker_ata_a).unwrap();
        let taker_ata_b_acc_data = spl_token::state::Account::unpack(&taker_ata_b_acc.data).unwrap();
        assert_eq!(taker_ata_b_acc_data.amount, 10);
    }

    #[test]
    fn test_refund() {
        let (
            mut program,
            maker,
            _taker,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            _taker_ata_a,
            _taker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let (escrow, vault) = make_flow(
            &mut program,
            &payer,
            &maker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            associated_token_program,
            token_program,
            system_program,
        );

        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker: maker.pubkey(),
                mint_a,
                maker_ata_a,
                escrow,
                vault,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        let message = Message::new(&[refund_ix], Some(&maker.pubkey()));
        let blockhash = program.latest_blockhash();
        let tx = Transaction::new(&[&maker], message, blockhash);
        let result = program.send_transaction(tx).unwrap();

        msg!("Refund successful | CU: {} | Sig: {}", result.compute_units_consumed, result.signature);
    }
}
