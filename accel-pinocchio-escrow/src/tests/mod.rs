#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use litesvm::{LiteSVM, types::TransactionMetadata};
    use litesvm_token::{spl_token::{self, solana_program::{msg, rent::Rent, sysvar::SysvarId}}, CreateAssociatedTokenAccount, CreateMint, MintTo};
    
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use pinocchio_token::state::TokenAccount;

    const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    
    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, u64, u64, u8) {

        let mut svm = LiteSVM::new();
        let payer = Keypair::new();   // also maker
        let maker = Keypair::new();
        let taker = Keypair::new();

        svm
            .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm
            .airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");
        svm
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
        let so_path = PathBuf::from("/home/abduo/Q4_2025_Accel_Abduovv/week_3_pinocchio/accel-pinocchio-escrow/target/deploy/escrow.so");
        msg!("The path is!! {:?}", so_path);

        msg!("Maker pubkey: {:?}", &maker.pubkey());
        msg!("Taker pubkey: {:?}", &taker.pubkey());
    
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    
        svm.add_program(program_id(), &program_data);

        
        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint_a = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(&mut svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}", mint_b);

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
            .owner(&maker.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
            .owner(&maker.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
            .owner(&maker.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
            .owner(&maker.pubkey()).send().unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), maker.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow.0);

        let escrow_ata = spl_associated_token_account::get_associated_token_address(
            &escrow.0,  
            &mint_a     
        );
        msg!("Escrow ATA PDA: {}\n", escrow_ata);

        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let amount_to_receive: u64 = 100000000; 
        let amount_to_give: u64 = 500000000;    
        let bump: u8 = escrow.1;

        msg!("Bump: {}", bump);

        (svm, maker, taker, escrow.0, mint_a, mint_b, maker_ata_a, maker_ata_b, taker_ata_a, taker_ata_b, escrow_ata, system_program, token_program, associated_token_program, amount_to_receive, amount_to_give, bump)

        
    }

    pub fn make_instruction(
        mut svm: LiteSVM,
        maker: Keypair,
        escrow: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        maker_ata_a: Pubkey,
        escrow_ata: Pubkey,
        system_program: Pubkey,
        token_program: Pubkey,
        associated_token_program: Pubkey,
        amount_to_receive: u64,
        amount_to_give: u64,
        bump: u8
    ) -> (Keypair, Pubkey, Pubkey) {

        let make_data = [
            vec![0u8],              
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ].concat();
        let make_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(escrow_ata, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        (maker, escrow, escrow_ata)

    }


    #[test]
    pub fn test_make_instruction() {
                let (
            mut svm,
            maker,
            _taker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            _,
            _,
            _,
            escrow_ata,
            system_program,
            token_program,
            associated_token_program,
            amount_to_receive,
            amount_to_give,
            bump
        ) = setup();
        make_instruction(svm, maker, escrow, mint_a, mint_b, maker_ata_a, escrow_ata, system_program, token_program, associated_token_program, amount_to_receive, amount_to_give, bump);

    }


    #[test]
    pub fn test_take_instruction() {
        let (
            mut svm,
            _maker,
            taker,
            _escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            _escrow_ata,
            system_program,
            token_program,
            associated_token_program,
            amount_to_receive,
            amount_to_give,
            bump
        ) = setup();
        let (maker, escrow, escrow_ata) = make_instruction(
            svm.clone(), 
            _maker, 
            _escrow, 
            mint_a, 
            mint_b, 
            maker_ata_a, 
            _escrow_ata, 
            system_program, 
            token_program, 
            associated_token_program, // associated
            amount_to_receive, 
            amount_to_give, 
            bump
        );

        let take_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(escrow_ata, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: [].to_vec(),
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        msg!("\n\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);


        let maker_ata_b_account = svm.get_account(&maker_ata_b).unwrap();

        let token_account_data = unsafe { TokenAccount::from_bytes_unchecked(&maker_ata_b_account.data) };

        let balance = token_account_data.amount();
        assert_eq!(balance, amount_to_receive, "Maker did not receive their mint_b");
        
        let taker_ata_a_account = svm.get_account(&taker_ata_a).unwrap();

        let token_account_data = unsafe { TokenAccount::from_bytes_unchecked(&taker_ata_a_account.data) };

        let balance = token_account_data.amount();

        assert_eq!(balance, amount_to_give, "Taker did not receive their mint_a");

    }

    #[test]
    pub fn test_cancel_instruction() {
        let (
            mut svm,
            _maker,
            _,
            _escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            _,
            _,
            _,
            _escrow_ata,
            system_program,
            token_program,
            associated_token_program,
            amount_to_receive,
            amount_to_give,
            bump
        ) = setup();
        let (maker, escrow, escrow_ata) = make_instruction(svm.clone(), _maker, _escrow, mint_a, mint_b, maker_ata_a, _escrow_ata, system_program, token_program, associated_token_program, amount_to_receive, amount_to_give, bump);

        let cancel_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(escrow_ata, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: [].to_vec(),
        };

        let message = Message::new(&[cancel_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        msg!("\n\nCancel transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}