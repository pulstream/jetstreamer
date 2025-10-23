use carbon_core::instruction::InstructionDecoder;
use solana_pubkey::Pubkey;
use std::sync::Arc;

use crate::{
    Plugin, PluginFuture,
    utils::transformers::{TransactionMetadata, extract_instructions_with_metadata},
};
use clickhouse::Client;
use futures_util::FutureExt;
use jetstreamer_firehose::firehose::{BlockData, TransactionData};
use log::info;
use solana_message::VersionedMessage;
use {
    carbon_pumpfun_decoder::PumpfunDecoder,
    carbon_pumpfun_decoder::instructions::PumpfunInstruction,
};

#[derive(Debug, Clone)]
/// Simple plugin that checks if transactions contain a specific mint address.
pub struct MintTrackingPlugin {
    /// The mint address to check for
    pub mint_address: Pubkey,
}

impl MintTrackingPlugin {
    /// Creates a new MintTrackingPlugin for the specified mint address
    pub fn new(mint_address: Pubkey) -> Self {
        Self { mint_address }
    }
}

impl Plugin for MintTrackingPlugin {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "Mint Tracking"
    }

    #[inline(always)]
    fn on_transaction<'a>(
        &'a self,
        _thread_id: usize,
        _db: Option<Arc<Client>>,
        transaction: &'a TransactionData,
    ) -> PluginFuture<'a> {
        let mint_address = self.mint_address;
        async move {
            let message = &transaction.transaction.message;
            let (account_keys, instructions) = match message {
                VersionedMessage::Legacy(msg) => (&msg.account_keys, &msg.instructions),
                VersionedMessage::V0(msg) => (&msg.account_keys, &msg.instructions),
            };

            if instructions.is_empty() {
                return Ok(());
            }

            // Check if the mint address is involved in any instruction
            let mint_involved = account_keys.iter().any(|&key| key == mint_address);

            if mint_involved {
                info!(
                    "Found transaction with mint {}: signature={}, slot={}",
                    mint_address, transaction.signature, transaction.slot
                );

                // Create TransactionMetadata from transaction data
                let transaction_metadata = Arc::new(TransactionMetadata {
                    slot: transaction.slot,
                    signature: transaction.signature,
                    fee_payer: transaction.transaction.message.static_account_keys()[0],
                    meta: transaction.transaction_status_meta.clone(),
                    message: transaction.transaction.message.clone(),
                });

                // Extract instructions with metadata using the transformers module
                let instructions_with_metadata = extract_instructions_with_metadata(
                    &transaction_metadata,
                    &transaction.transaction.message,
                    &transaction.transaction_status_meta,
                );

                // Process each instruction
                let decoder = PumpfunDecoder;
                for (_metadata, instruction) in instructions_with_metadata {
                    if let Some(decoded) = decoder.decode_instruction(&instruction) {
                        match decoded.data {
                            PumpfunInstruction::TradeEvent(trade_event) => {
                                info!(
                                    "🎯 TARGET MINT - Signature: {}, Slot: {}, Decoded Pumpfun Trade Event - Mint: {}, User: {}, Amount In: {}, Amount Out: {}, Is Buy: {}, Timestamp: {}",
                                    transaction.signature,
                                    transaction.slot,
                                    trade_event.mint,
                                    trade_event.user,
                                    if trade_event.is_buy { trade_event.sol_amount } else { trade_event.token_amount },
                                    if trade_event.is_buy { trade_event.token_amount } else { trade_event.sol_amount },
                                    trade_event.is_buy,
                                    trade_event.timestamp
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(())
        }
        .boxed()
    }

    #[inline(always)]
    fn on_block(
        &self,
        _thread_id: usize,
        _db: Option<Arc<Client>>,
        _block: &BlockData,
    ) -> PluginFuture<'_> {
        async move { Ok(()) }.boxed()
    }

    #[inline(always)]
    fn on_load(&self, _db: Option<Arc<Client>>) -> PluginFuture<'_> {
        let mint_address = self.mint_address;
        async move {
            info!("Mint Tracking Plugin loaded for mint: {}", mint_address);
            Ok(())
        }
        .boxed()
    }

    #[inline(always)]
    fn on_exit(&self, _db: Option<Arc<Client>>) -> PluginFuture<'_> {
        async move { Ok(()) }.boxed()
    }
}
