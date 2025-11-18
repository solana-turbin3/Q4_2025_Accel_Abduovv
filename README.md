<img width="1200" height="630" alt="image" src="https://github.com/user-attachments/assets/9fcac023-dfab-46c3-ab1c-9777b3fe7131" />

![Powered by Solana](https://img.shields.io/badge/Powered%20by-Solana-00FFA3?logo=solana&logoColor=white)  ![Anchor](https://img.shields.io/badge/Anchor-Framework-blueviolet) ![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white) ![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)
 ![LiteSVM](https://img.shields.io/badge/LiteSVM-lightgrey?logo=rust&logoColor=black)

---

# ⚡ Turbin3 Accelerate Builders – Q4 2025 Cohort  

Welcome to my **Proof of Work repository** for the **Turbin3 Accelerate Builders Program**.  
This repo chronicles my weekly progress through hands-on Solana development challenges — from SPL Token extensions and on-chain hooks to high-performance testing and advanced protocol design.  

---

## 🧭 Repository Overview  

```cpp
Q4_2025_Accel_Abduovv/
├── week_1/  // Week 1: SPL Token 2022 + LiteSVM
│ ├── whitelist-transfer-hook/ // Whitelist-based Transfer Hook
│ └── escrow-litesvm/ // Escrow with time-lock & LiteSVM tests
└── README.md
```

---

## 🧩 Week 1 — Token Extensions & LiteSVM  

**Theme:** Exploring SPL Token-2022 extensions and building ultra-fast test environments with LiteSVM.  
This week’s focus was on designing modular, composable Solana programs with practical security patterns.

---

### 🧱 Project 1: Whitelist Transfer Hook  

A **SPL Token-2022 Transfer Hook** enforcing whitelist-based transfer permissions.  
Each user has a **dedicated PDA whitelist account**, ensuring scalable and rent-optimized access control.  

**🔧 Key Work Completed**
- ✅ Replaced array-based whitelist with **per-user PDA** (`[b"whitelist", user]`) — scalable and reallocation-free  
- ✅ Programmatically **initialized the mint** with the Transfer Hook on-chain, eliminating client-side dependency
---

### 🔐 Project 2: Escrow (LiteSVM)  

A **trustless token escrow** for atomic swaps between two parties, secured by **time-lock logic** and tested with **LiteSVM** for 10× faster feedback cycles.  

**🔧 Key Work Completed**
- ✅ Added full test coverage for `take` and `refund` instructions  
- ✅ Integrated **time-lock protection** using `Clock::get()?.unix_timestamp`, with strict unlock logic and unit tests  

**💡 Core Features**
- Three main flows: `make`, `take`, `refund`  
- Vault PDA for secure token custody  
- Time-locked trades using Unix timestamps  
- Deterministic LiteSVM testing (fast, local, reproducible)  

**🧪 Test Cases**
- `test_make` → Initializes escrow and validates state  
- `test_take` → Executes valid swap  
- `test_refund` → Returns tokens to maker  

---

## 🔜 Week 2 — Ephemeral Rollups, RPCs & Indexing  


---

## 🔜 Week 3 — Project “Pinocchio”  


---

## 🔜 Week 4 — MPL + Codama + Group  
