mod constant;
#[allow(unused)]
mod pb;

use crate::constant::{
    DAMM_SWAP, DAMM_SWAP2, DBC_SWAP, DBC_SWAP2, DLMM_SWAP, DLMM_SWAP2, DLMM_SWAP_EXACT_OUT,
    DLMM_SWAP_EXACT_OUT2, DLMM_SWAP_WITH_PRICE_IMPACT, DLMM_SWAP_WITH_PRICE_IMPACT2,
    FILTER_PROGRAM_IDS,
};
use pb::meteora::{SwapEvent, SwapEvents, SwapSide};
use substreams_solana::b58;
use substreams_solana::pb::sf::solana::r#type::v1::{
    Block, ConfirmedTransaction, Message, TransactionStatusMeta,
};
use substreams_solana_utils::instruction::{get_flattened_instructions, WrappedInstruction};
use substreams_solana_utils::pubkey::Pubkey;
use substreams_solana_utils::spl_token::TOKEN_PROGRAM_ID;

const TOKEN_2022_PROGRAM_ID: Pubkey =
    Pubkey(b58!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"));

// ============================================================================
// 主 Map Handler
// ============================================================================
#[substreams::handlers::map]
fn meteora(block: Block) -> SwapEvents {
    let mut swap_events = SwapEvents::default();
    let slot = block.slot;
    let timestamp_ms = block
        .block_time
        .as_ref()
        .map(|t| t.timestamp as u64 * 1000)
        .unwrap_or(0);

    for (tx_index, tx) in block.transactions.iter().enumerate() {
        // 跳过失败的交易
        if let Some(ref meta) = tx.meta {
            if meta.err.is_some() {
                continue;
            }
        } else {
            continue;
        }

        // 解析交易中的 Swap 事件
        if let Some(events) = parse_transaction(tx, slot, timestamp_ms, tx_index as u32) {
            swap_events.swaps.extend(events);
        }
    }

    swap_events
}

// ============================================================================
// 解析单笔交易
// ============================================================================
fn parse_transaction(
    tx: &ConfirmedTransaction,
    slot: u64,
    timestamp_ms: u64,
    tx_index: u32,
) -> Option<Vec<SwapEvent>> {
    let transaction = tx.transaction.as_ref()?;
    let message = transaction.message.as_ref()?;
    let meta = tx.meta.as_ref()?;

    // 获取交易签名
    let signature = bs58::encode(&transaction.signatures[0]).into_string();

    // 获取发起者 (第一个签名者)
    let signer = get_account_key(message, meta, 0)?;
    let signer_str = bs58::encode(&signer).into_string();

    // 获取完整账户列表 (包含 ALT 补充地址)
    let account_keys = resolved_account_keys(message, Some(meta));

    let mut swap_events = Vec::new();

    let instructions = get_flattened_instructions(tx);

    // 遍历所有指令 (包括内部指令)
    for (inst_index, inst) in instructions.iter().enumerate() {
        let program_id_index = inst.program_id_index() as usize;

        // 获取程序 ID，如果索引越界则跳过
        let program_id = match account_keys.get(program_id_index) {
            Some(id) => id,
            None => continue,
        };

        // 检查是否是目标程序，不是则跳过
        let platform = match match_program(program_id) {
            Some(p) => p,
            None => continue,
        };

        // 获取指令数据
        let data = inst.data();
        if data.len() < 8 {
            continue;
        }

        // 匹配 Swap Discriminator
        let discriminator: [u8; 8] = match data[0..8].try_into() {
            Ok(d) => d,
            Err(_) => continue,
        };
        if !is_swap_discriminator(&discriminator, &platform) {
            continue;
        }

        // 获取指令的 accounts 列表
        let inst_accounts = inst.accounts();

        // 提取 Pool 地址
        let pool_index = get_pool_account_index(&platform);
        let pool = inst_accounts
            .get(pool_index)
            .and_then(|&idx| account_keys.get(idx as usize))
            .map(|k| bs58::encode(k).into_string())
            .unwrap_or_default();

        // 获取用户 Token 账户的 account_index (用于在 TokenBalance 中查找)
        let (input_acc_idx, output_acc_idx) = get_user_token_account_indices(&platform);

        let input_account_idx = inst_accounts.get(input_acc_idx).copied();
        let output_account_idx = inst_accounts.get(output_acc_idx).copied();

        // 优先使用当前 swap instruction 的 inner instruction (token transfer) 计算金额，
        // 避免多跳/多次 swap 导致的「整笔交易余额变化」误差。
        let (base_mint, quote_mint, base_amount, quote_amount, base_decimals, quote_decimals, side) =
            match extract_swap_amounts_by_inner_transfers(
                &instructions,
                inst_index,
                meta,
                &account_keys,
                input_account_idx,
                output_account_idx,
            )
            .or_else(|| {
                extract_swap_amounts_by_accounts(
                    meta,
                    &signer_str,
                    input_account_idx.map(|x| x as u32),
                    output_account_idx.map(|x| x as u32),
                )
            }) {
                Some((bm, qm, ba, qa, bd, qd, s)) if bm != qm => (bm, qm, ba, qa, bd, qd, s),
                _ => {
                    // 找不到金额或 mint 相同时，输出占位事件 (金额为 0)
                    (
                        String::new(),
                        String::new(),
                        0,
                        0,
                        0,
                        0,
                        SwapSide::SideUnknown,
                    )
                }
            };

        swap_events.push(SwapEvent {
            pool,
            signature: signature.clone(),
            user: signer_str.clone(),
            platform: platform.to_string(),
            timestamp_ms,
            slot,
            tx_index,
            base_mint,
            quote_mint,
            base_amount,
            quote_amount,
            side: side.into(),
            base_decimals,
            quote_decimals,
        });
    }

    if swap_events.is_empty() {
        None
    } else {
        Some(swap_events)
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 匹配程序 ID，返回平台名称
fn match_program(program_id: &[u8]) -> Option<&'static str> {
    for (target_id, platform_name) in FILTER_PROGRAM_IDS.iter() {
        if target_id.0.as_slice() == program_id {
            return Some(*platform_name);
        }
    }
    None
}

/// 检查是否是 Swap 类型的 Discriminator
fn is_swap_discriminator(discriminator: &[u8; 8], platform: &str) -> bool {
    match platform {
        "meteora_dlmm" => {
            discriminator == &DLMM_SWAP
                || discriminator == &DLMM_SWAP2
                || discriminator == &DLMM_SWAP_EXACT_OUT
                || discriminator == &DLMM_SWAP_EXACT_OUT2
                || discriminator == &DLMM_SWAP_WITH_PRICE_IMPACT
                || discriminator == &DLMM_SWAP_WITH_PRICE_IMPACT2
        }
        "meteora_damm" => discriminator == &DAMM_SWAP || discriminator == &DAMM_SWAP2,
        "meteora_dbc" => discriminator == &DBC_SWAP || discriminator == &DBC_SWAP2,
        _ => false,
    }
}

/// 获取 Pool 地址在 accounts 中的索引 (根据 IDL)
fn get_pool_account_index(platform: &str) -> usize {
    match platform {
        "meteora_dlmm" => 0, // lb_pair 在 accounts[0]
        "meteora_damm" => 1, // pool 在 accounts[1], accounts[0] 是 pool_authority
        "meteora_dbc" => 2, // pool 在 accounts[2], accounts[0] 是 pool_authority, accounts[1] 是 config
        _ => 0,
    }
}

/// 获取用户 Token 输入/输出账户在 accounts 中的索引 (根据 IDL)
/// 返回: (input_token_account_index, output_token_account_index)
fn get_user_token_account_indices(platform: &str) -> (usize, usize) {
    match platform {
        // DLMM: user_token_in (4), user_token_out (5)
        "meteora_dlmm" => (4, 5),
        // DAMM: input_token_account (2), output_token_account (3)
        "meteora_damm" => (2, 3),
        // DBC: input_token_account (3), output_token_account (4)
        "meteora_dbc" => (3, 4),
        _ => (0, 0),
    }
}

fn token_balance_mint_and_decimals(
    meta: &TransactionStatusMeta,
    account_index: u32,
) -> Option<(String, u32)> {
    let b = meta
        .post_token_balances
        .iter()
        .find(|b| b.account_index == account_index)
        .or_else(|| meta.pre_token_balances.iter().find(|b| b.account_index == account_index))?;

    let decimals = b
        .ui_token_amount
        .as_ref()
        .map(|a| a.decimals)
        .unwrap_or(0);

    Some((b.mint.clone(), decimals))
}

fn parse_transfer_amount_with_token2022(
    program_id: &[u8],
    accounts: &[u8],
    data: &[u8],
) -> Option<(u8, u8, u64, Option<u8>, Option<u8>)> {
    if program_id != TOKEN_PROGRAM_ID.0.as_slice() && program_id != TOKEN_2022_PROGRAM_ID.0.as_slice()
    {
        return None;
    }

    let tag = *data.first()?;
    match tag {
        // Transfer
        3 => {
            if data.len() < 9 || accounts.len() < 2 {
                return None;
            }
            let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);
            Some((accounts[0], accounts[1], amount, None, None))
        }
        // TransferChecked
        12 => {
            if data.len() < 10 || accounts.len() < 3 {
                return None;
            }
            let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);
            let decimals = data.get(9).copied();
            // accounts: [source, mint, destination, authority, ...]
            Some((accounts[0], accounts[2], amount, Some(accounts[1]), decimals))
        }
        _ => None,
    }
}

fn extract_swap_amounts_by_inner_transfers<'a>(
    instructions: &[WrappedInstruction<'a>],
    swap_instruction_index: usize,
    meta: &TransactionStatusMeta,
    account_keys: &[Vec<u8>],
    input_account_idx: Option<u8>,
    output_account_idx: Option<u8>,
) -> Option<(String, String, u64, u64, u32, u32, SwapSide)> {
    let input_account_idx = input_account_idx?;
    let output_account_idx = output_account_idx?;

    // 如果缺少 stack_height（老版本 Solana），无法可靠地关联「某条指令」对应的 inner instructions
    let swap_height = instructions.get(swap_instruction_index)?.stack_height()?;

    let mut input_amount: u64 = 0;
    let mut output_amount: u64 = 0;
    let mut input_mint: Option<String> = None;
    let mut output_mint: Option<String> = None;
    let mut input_decimals: Option<u32> = None;
    let mut output_decimals: Option<u32> = None;

    // 在扁平化指令流中，收集该 swap instruction 的所有后代指令 (stack_height > swap_height)
    let mut i = swap_instruction_index + 1;
    while i < instructions.len() {
        let inst = &instructions[i];
        let inst_height = match inst.stack_height() {
            Some(h) => h,
            None => break,
        };
        if inst_height <= swap_height {
            break;
        }

        let program_id = match account_keys.get(inst.program_id_index() as usize) {
            Some(id) => id.as_slice(),
            None => {
                i += 1;
                continue;
            }
        };

        if let Some((source, destination, amount, mint_idx, decimals)) =
            parse_transfer_amount_with_token2022(program_id, inst.accounts(), inst.data())
        {
            if source == input_account_idx {
                input_amount = input_amount.saturating_add(amount);
                if input_mint.is_none() {
                    input_mint = mint_idx
                        .and_then(|idx| account_keys.get(idx as usize))
                        .map(|k| bs58::encode(k).into_string())
                        .or_else(|| token_balance_mint_and_decimals(meta, input_account_idx as u32).map(|x| x.0));
                }
                if input_decimals.is_none() {
                    input_decimals = decimals
                        .map(|d| d as u32)
                        .or_else(|| token_balance_mint_and_decimals(meta, input_account_idx as u32).map(|x| x.1));
                }
            }
            if destination == output_account_idx {
                output_amount = output_amount.saturating_add(amount);
                if output_mint.is_none() {
                    output_mint = mint_idx
                        .and_then(|idx| account_keys.get(idx as usize))
                        .map(|k| bs58::encode(k).into_string())
                        .or_else(|| token_balance_mint_and_decimals(meta, output_account_idx as u32).map(|x| x.0));
                }
                if output_decimals.is_none() {
                    output_decimals = decimals
                        .map(|d| d as u32)
                        .or_else(|| token_balance_mint_and_decimals(meta, output_account_idx as u32).map(|x| x.1));
                }
            }
        }

        i += 1;
    }

    if input_amount == 0 || output_amount == 0 {
        return None;
    }

    let input_mint = input_mint?;
    let output_mint = output_mint?;
    let input_decimals = input_decimals.unwrap_or(0);
    let output_decimals = output_decimals.unwrap_or(0);

    if input_mint.is_empty() || output_mint.is_empty() || input_mint == output_mint {
        return None;
    }

    const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
    if input_mint == WSOL_MINT && output_mint != WSOL_MINT {
        // Buy: spend SOL/WSOL to receive token
        Some((
            output_mint,
            input_mint,
            output_amount,
            input_amount,
            output_decimals,
            input_decimals,
            SwapSide::SideBuy,
        ))
    } else if output_mint == WSOL_MINT && input_mint != WSOL_MINT {
        // Sell: spend token to receive SOL/WSOL
        Some((
            input_mint,
            output_mint,
            input_amount,
            output_amount,
            input_decimals,
            output_decimals,
            SwapSide::SideSell,
        ))
    } else {
        // Fallback: token-token swap (unknown base/quote)
        Some((
            output_mint,
            input_mint,
            output_amount,
            input_amount,
            output_decimals,
            input_decimals,
            SwapSide::SideUnknown,
        ))
    }
}

/// 根据指令的输入/输出账户索引，从 Token Balance 变化中提取 Swap 金额
/// 同时处理临时 WSOL 账户的情况（通过 Native SOL 余额变化）
/// signer: 交易签名者地址，只统计 owner == signer 的 Token 变化
fn extract_swap_amounts_by_accounts(
    meta: &TransactionStatusMeta,
    signer: &str,
    input_account_idx: Option<u32>,
    output_account_idx: Option<u32>,
) -> Option<(String, String, u64, u64, u32, u32, SwapSide)> {
    let pre_balances = &meta.pre_token_balances;
    let post_balances = &meta.post_token_balances;

    // WSOL mint 地址
    const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

    // 首先尝试通过指定的账户索引查找
    if let (Some(input_idx), Some(output_idx)) = (input_account_idx, output_account_idx) {
        let input_change = find_token_balance_change(pre_balances, post_balances, input_idx);
        let output_change = find_token_balance_change(pre_balances, post_balances, output_idx);

        if let (
            Some((input_mint, input_amount, input_decimals)),
            Some((output_mint, output_amount, output_decimals)),
        ) = (input_change, output_change)
        {
            // 输入应该是负数 (用户付出)，输出应该是正数 (用户收到)
            if input_amount < 0 && output_amount > 0 {
                return Some((
                    output_mint,            // base_mint (收到的)
                    input_mint,             // quote_mint (付出的)
                    output_amount as u64,   // base_amount
                    (-input_amount) as u64, // quote_amount
                    output_decimals,        // base_decimals
                    input_decimals,         // quote_decimals
                    SwapSide::SideBuy,      // 用 quote 买 base
                ));
            }
        }
    }

    // 收集签名者拥有的 Token Balance 变化 (只统计 owner == signer)
    let mut changes: Vec<(String, i64, u32)> = Vec::new();

    for post in post_balances.iter() {
        // 只统计签名者的 Token 变化
        if post.owner != signer {
            continue;
        }

        let account_idx = post.account_index;
        let pre_amount: u64 = pre_balances
            .iter()
            .find(|b| b.account_index == account_idx)
            .and_then(|b| b.ui_token_amount.as_ref())
            .and_then(|amt| amt.amount.parse().ok())
            .unwrap_or(0);

        let post_amount: u64 = post
            .ui_token_amount
            .as_ref()
            .and_then(|amt| amt.amount.parse().ok())
            .unwrap_or(0);

        let change = post_amount as i64 - pre_amount as i64;
        if change != 0 {
            let mint = post.mint.clone();
            let decimals = post
                .ui_token_amount
                .as_ref()
                .map(|a| a.decimals)
                .unwrap_or(0);
            changes.push((mint, change, decimals));
        }
    }

    // 方案 1: 找到一个负变化和一个正变化（两个不同 Token）
    let negative = changes.iter().find(|(_, c, _)| *c < 0);
    let positive = changes.iter().find(|(_, c, _)| *c > 0);

    if let (
        Some((quote_mint, quote_change, quote_decimals)),
        Some((base_mint, base_change, base_decimals)),
    ) = (negative, positive)
    {
        if quote_mint != base_mint {
            return Some((
                base_mint.clone(),
                quote_mint.clone(),
                *base_change as u64,
                (-*quote_change) as u64,
                *base_decimals,
                *quote_decimals,
                SwapSide::SideBuy,
            ));
        }
    }

    // 方案 2: 只有一个 Token 变化 + Native SOL 变化（临时 WSOL 场景）
    // 计算用户的 Native SOL 变化 (preBalances[0] - postBalances[0] - fee)
    if !meta.pre_balances.is_empty() && !meta.post_balances.is_empty() {
        let pre_sol = meta.pre_balances[0] as i64;
        let post_sol = meta.post_balances[0] as i64;
        let fee = meta.fee as i64;

        // 用户实际的 SOL 变化（排除 gas 费用）
        let sol_change = post_sol - pre_sol + fee; // 负数 = 付出，正数 = 收到

        // 如果有显著的 SOL 变化（超过 1000 lamports 以排除噪音）
        if sol_change.abs() > 1000 {
            // 场景 A: 用户付出 SOL，收到 Token (Buy)
            if sol_change < 0 && positive.is_some() && negative.is_none() {
                let (base_mint, base_change, base_decimals) = positive.unwrap();
                return Some((
                    base_mint.clone(),
                    WSOL_MINT.to_string(),
                    *base_change as u64,
                    (-sol_change) as u64,
                    *base_decimals,
                    9, // SOL decimals
                    SwapSide::SideBuy,
                ));
            }

            // 场景 B: 用户付出 Token，收到 SOL (Sell)
            if sol_change > 0 && negative.is_some() && positive.is_none() {
                let (quote_mint, quote_change, quote_decimals) = negative.unwrap();
                return Some((
                    WSOL_MINT.to_string(),
                    quote_mint.clone(),
                    sol_change as u64,
                    (-*quote_change) as u64,
                    9, // SOL decimals
                    *quote_decimals,
                    SwapSide::SideSell,
                ));
            }
        }
    }

    None
}

/// 查找特定账户的 Token Balance 变化
/// 返回: (mint, change, decimals)
fn find_token_balance_change(
    pre_balances: &[substreams_solana::pb::sf::solana::r#type::v1::TokenBalance],
    post_balances: &[substreams_solana::pb::sf::solana::r#type::v1::TokenBalance],
    account_index: u32,
) -> Option<(String, i64, u32)> {
    // 在 post_balances 中查找
    let post = post_balances
        .iter()
        .find(|b| b.account_index == account_index);

    let post_amount: u64 = post
        .and_then(|b| b.ui_token_amount.as_ref())
        .and_then(|amt| amt.amount.parse().ok())
        .unwrap_or(0);

    let pre_amount: u64 = pre_balances
        .iter()
        .find(|b| b.account_index == account_index)
        .and_then(|b| b.ui_token_amount.as_ref())
        .and_then(|amt| amt.amount.parse().ok())
        .unwrap_or(0);

    let mint = post.map(|b| b.mint.clone()).unwrap_or_default();
    let decimals = post
        .and_then(|b| b.ui_token_amount.as_ref())
        .map(|a| a.decimals)
        .unwrap_or(0);

    let change = post_amount as i64 - pre_amount as i64;

    if change == 0 && mint.is_empty() {
        return None;
    }

    Some((mint, change, decimals))
}

/// 获取账户地址 (支持 ALT)
fn get_account_key(
    message: &Message,
    meta: &TransactionStatusMeta,
    index: usize,
) -> Option<Vec<u8>> {
    let keys = resolved_account_keys(message, Some(meta));
    keys.get(index).cloned()
}

/// 合并 message 中的 account_keys 与 ALT 补充地址
fn resolved_account_keys(message: &Message, meta: Option<&TransactionStatusMeta>) -> Vec<Vec<u8>> {
    let mut keys = message.account_keys.clone();

    if let Some(meta) = meta {
        if !meta.loaded_writable_addresses.is_empty() {
            keys.extend(meta.loaded_writable_addresses.clone());
        }
        if !meta.loaded_readonly_addresses.is_empty() {
            keys.extend(meta.loaded_readonly_addresses.clone());
        }
    }

    keys
}
