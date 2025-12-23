use substreams_solana::b58;
use substreams_solana_utils::pubkey::Pubkey;

// ============================================================================
// Meteora Program IDs
// ============================================================================

/// DLMM (Liquidity Book) - 集中流动性做市
pub const METEORA_DLMM_PROGRAM_ID: Pubkey =
    Pubkey(b58!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"));

/// DAMM V2 (Dynamic AMM) - 动态自动做市商
pub const METEORA_DAMM_V2_PROGRAM_ID: Pubkey =
    Pubkey(b58!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG"));

/// DBC (Dynamic Bonding Curve) - 动态绑定曲线
pub const METEORA_DBC_PROGRAM_ID: Pubkey =
    Pubkey(b58!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN"));

/// 程序 ID 与平台名称映射表
pub const FILTER_PROGRAM_IDS: &[(Pubkey, &str)] = &[
    (METEORA_DLMM_PROGRAM_ID, "meteora_dlmm"),
    (METEORA_DAMM_V2_PROGRAM_ID, "meteora_damm"),
    (METEORA_DBC_PROGRAM_ID, "meteora_dbc"),
];

// ============================================================================
// DLMM Swap Discriminators (LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo)
// Pool 地址: accounts[0] (lb_pair)
// ============================================================================

/// swap - 基础 Swap 方法
/// args: amount_in (u64), min_amount_out (u64)
pub const DLMM_SWAP: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// swap2 - Swap 改进版，支持 memo 和更多 bin
/// args: amount_in (u64), min_amount_out (u64), remaining_accounts_info
pub const DLMM_SWAP2: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

/// swap_exact_out - 精确输出金额的 Swap
/// args: max_in_amount (u64), out_amount (u64)
pub const DLMM_SWAP_EXACT_OUT: [u8; 8] = [250, 73, 101, 33, 38, 207, 75, 184];

/// swap_exact_out2 - 精确输出改进版
/// args: max_in_amount (u64), out_amount (u64), remaining_accounts_info
pub const DLMM_SWAP_EXACT_OUT2: [u8; 8] = [43, 215, 247, 132, 137, 60, 243, 81];

/// swap_with_price_impact - 带价格影响保护的 Swap
/// args: amount_in (u64), active_id (Option<i32>), max_price_impact_bps (u16)
pub const DLMM_SWAP_WITH_PRICE_IMPACT: [u8; 8] = [56, 173, 230, 208, 173, 228, 156, 205];

/// swap_with_price_impact2 - 价格影响保护改进版
/// args: amount_in (u64), active_id (Option<i32>), max_price_impact_bps (u16), remaining_accounts_info
pub const DLMM_SWAP_WITH_PRICE_IMPACT2: [u8; 8] = [74, 98, 192, 214, 177, 51, 75, 51];

// ============================================================================
// DAMM V2 Swap Discriminators (cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG)
// Pool 地址: accounts[1] (pool), accounts[0] 是固定的 pool_authority
// ============================================================================

/// swap - DAMM V2 基础 Swap 方法
/// args: params (SwapParameters)
/// 注意: discriminator 与 DLMM swap 相同 (Anchor 方法名相同)
pub const DAMM_SWAP: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// swap2 - DAMM V2 Swap 改进版
/// args: params (SwapParameters2)
/// 注意: discriminator 与 DLMM swap2 相同 (Anchor 方法名相同)
pub const DAMM_SWAP2: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

// ============================================================================
// DBC Swap Discriminators (dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN)
// Pool 地址: accounts[2] (pool), accounts[0] 是 pool_authority, accounts[1] 是 config
// ============================================================================

/// swap - DBC 基础 Swap 方法 (用于 Trading Bots)
/// args: params (SwapParameters)
/// 注意: discriminator 与 DLMM/DAMM swap 相同 (Anchor 方法名相同)
pub const DBC_SWAP: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// swap2 - DBC Swap 改进版
/// args: params (SwapParameters2)
/// 注意: discriminator 与 DLMM/DAMM swap2 相同 (Anchor 方法名相同)
pub const DBC_SWAP2: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
