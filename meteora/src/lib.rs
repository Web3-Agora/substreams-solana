mod constant;
#[allow(unused)]
mod pb;

use crate::constant::FILTER_PROGRAM_IDS;

use pb::meteora::Meteora;
use substreams_solana::pb::sf::solana::r#type::v1::{
    Block, ConfirmedTransaction, Message, TransactionStatusMeta,
};
use substreams_solana_utils::instruction::get_flattened_instructions;



// 主 map 处理：从 blocks_without_votes 过滤出匹配程序 ID 的交易并打包输出
#[substreams::handlers::map]
fn meteora(block: Block) -> Meteora {
    let mut my_data = Meteora::default();
    for tx in block.transactions.into_iter() {
        if should_keep_transaction(&tx) {
            my_data.transactions.push(tx);
        }
    }

    my_data
}

// 判断交易是否包含目标程序 ID 的指令
fn should_keep_transaction(tx: &ConfirmedTransaction) -> bool {
    let (message, meta) = match (
        tx.transaction.as_ref().and_then(|t| t.message.as_ref()),
        tx.meta.as_ref(),
    ) {
        (Some(message), Some(meta)) => (message, meta),
        _ => return false,
    };

    let account_keys = resolved_account_keys(message, Some(meta));

    let matches_filter = |program_id_index: u32| -> bool {
        account_keys
            .get(program_id_index as usize)
            .map(|program_key| {
                FILTER_PROGRAM_IDS
                    .iter()
                    .any(|target| target.0.as_slice() == program_key.as_slice())
            })
            .unwrap_or(false)
    };

    for inst in get_flattened_instructions(tx) {
        if matches_filter(inst.program_id_index()) {
            return true;
        }
    }

    false
}

// 合并 message 中的 account_keys 与 ALT 补充出的地址列表
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


