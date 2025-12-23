use substreams_solana_utils::pubkey::Pubkey;
use substreams_solana::b58;

// Meteora program IDs
pub const METEORA_DLMM_PROGRAM_ID: Pubkey = Pubkey(b58!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"));
pub const METEORA_DAMM_V2_PROGRAM_ID: Pubkey = Pubkey(b58!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG"));
pub const METEORA_DBC_PROGRAM_ID: Pubkey = Pubkey(b58!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN"));

// 汇总便于遍历/去重
pub const FILTER_PROGRAM_IDS: &[Pubkey] = &[
    METEORA_DLMM_PROGRAM_ID,
    METEORA_DAMM_V2_PROGRAM_ID,
    METEORA_DBC_PROGRAM_ID,
];
