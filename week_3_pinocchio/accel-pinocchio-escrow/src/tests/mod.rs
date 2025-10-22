#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use litesvm::LiteSVM;
    use litesvm_token::{spl_token::{self, solana_program::{msg, rent::Rent, sysvar::SysvarId}}, CreateAssociatedTokenAccount, CreateMint, MintTo};
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    
    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (
        LiteSVM, Keypair, Keypair, Keypair, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, Pubkey, (Pubkey, u8), Pubkey, Pubkey, Pubkey, Pubkey
    ) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        let maker = Keypair::new();
        let taker = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let so_path = PathBuf::from("/home/abduo/Q4_2025_Accel_Abduovv/week_3_pinocchio/accel-pinocchio-escrow/target/sbpf-solana-solana/release/escrow.so");
        let program_data = std::fs::read(so_path).unwrap();
        svm.add_program(program_id(), &program_data);

        let mint_a = CreateMint::new(&mut svm, &payer).decimals(6).authority(&payer.pubkey()).send().unwrap();
        let mint_b = CreateMint::new(&mut svm, &payer).decimals(6).authority(&payer.pubkey()).send().unwrap();

        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a).owner(&maker.pubkey()).send().unwrap();
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b).owner(&maker.pubkey()).send().unwrap();
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a).owner(&taker.pubkey()).send().unwrap();
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b).owner(&taker.pubkey()).send().unwrap();


        let escrow = Pubkey::find_program_address(&[b"escrow".as_ref(), maker.pubkey().as_ref()], &PROGRAM_ID.parse().unwrap());
        let escrow_ata = spl_associated_token_account::get_associated_token_address(&escrow.0, &mint_a);
        // let escrow_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a).owner(&escrow.0).send().unwrap();
    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000).send().unwrap();

        (
            svm, payer, maker, taker, mint_a, mint_b, maker_ata_a, maker_ata_b, taker_ata_a, taker_ata_b, escrow, escrow_ata, associated_token_program, token_program, system_program
        )
    }

    pub fn make_instruction(
        svm: &LiteSVM, maker: &Keypair, mint_a: Pubkey, mint_b: Pubkey, maker_ata_a: Pubkey,
        escrow: (Pubkey, u8), escrow_ata: Pubkey, associated_token_program: Pubkey, token_program: Pubkey, system_program: Pubkey
    ) -> (Pubkey, Pubkey) {
        let program_id = program_id();
        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let amount_to_receive: u64 = 100000000;
        let amount_to_give: u64 = 500000000;
        let bump: u8 = escrow.1;

        let make_data = [vec![0u8], bump.to_le_bytes().to_vec(), amount_to_receive.to_le_bytes().to_vec(), amount_to_give.to_le_bytes().to_vec()].concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow.0, false),
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
        svm.clone().send_transaction(transaction).unwrap();

        (escrow.0, escrow_ata)
    }

    #[test]
    pub fn test_make_instruction() {
        let (svm, _payer, maker, _taker, mint_a, mint_b, maker_ata_a, _maker_ata_b, _taker_ata_a, _taker_ata_b, escrow, escrow_ata, associated_token_program, token_program, system_program) = setup();
        make_instruction(&svm, &maker, mint_a, mint_b, maker_ata_a, escrow, escrow_ata, associated_token_program, token_program, system_program);
    }

    #[test]
    pub fn test_take_instruction() {
        let (mut svm, _payer, maker, taker, mint_a, mint_b, maker_ata_a, maker_ata_b, taker_ata_a, taker_ata_b, _escrow, _escrow_ata, associated_token_program, token_program, system_program) = setup();
      let (escrow, escrow_ata) = make_instruction(&svm, &maker, mint_a, mint_b, maker_ata_a, _escrow, _escrow_ata, associated_token_program, token_program, system_program);

        let program_id: Pubkey = program_id();
        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let take_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(mint_a, false),
                AccountMeta::new(mint_b, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(taker_ata_b, false),
                AccountMeta::new(taker_ata_a, false),
                AccountMeta::new(maker_ata_b, false),
                AccountMeta::new(escrow_ata, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(token_program, false),
                AccountMeta::new(associated_token_program, false),
            ],
            data: [].to_vec(),
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction = Transaction::new(&[&taker], message, recent_blockhash);
        svm.send_transaction(transaction).unwrap();
    }

    #[test]
    pub fn test_cancel_instruction() {
        let (mut svm, _payer, maker, _taker, mint_a, mint_b, maker_ata_a, _maker_ata_b, _taker_ata_a, _taker_ata_b, _escrow, _escrow_ata, associated_token_program, token_program, system_program) = setup();
      let (escrow, escrow_ata) = make_instruction(&svm, &maker, mint_a, mint_b, maker_ata_a, _escrow, _escrow_ata, associated_token_program, token_program, system_program);

        let program_id: Pubkey = program_id();
        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let cancel_ix = Instruction {
            program_id: program_id,
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
        svm.send_transaction(transaction).unwrap();
    }
}
