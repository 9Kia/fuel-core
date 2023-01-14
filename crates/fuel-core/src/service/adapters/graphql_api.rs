use crate::{
    database::{
        transactions::OwnedTransactionIndexCursor,
        Database,
    },
    fuel_core_graphql_api::ports::{
        DatabaseBlocks,
        DatabaseChain,
        DatabaseCoins,
        DatabaseContracts,
        DatabaseMessages,
        DatabasePort,
        DatabaseTransactions,
    },
    state::IterDirection,
};
use fuel_core_storage::{
    iter::{
        BoxedIter,
        IntoBoxedIter,
    },
    not_found,
    Error as StorageError,
    Result as StorageResult,
};
use fuel_core_txpool::types::{
    ContractId,
    TxId,
};
use fuel_core_types::{
    blockchain::primitives::{
        BlockHeight,
        BlockId,
        DaBlockHeight,
    },
    entities::message::Message,
    fuel_tx::{
        Address,
        AssetId,
        MessageId,
        TxPointer,
        UtxoId,
    },
    services::{
        graphql_api::ContractBalance,
        txpool::TransactionStatus,
    },
};

impl DatabaseBlocks for Database {
    fn block_id(&self, height: BlockHeight) -> StorageResult<BlockId> {
        self.get_block_id(height)
            .and_then(|heigh| heigh.ok_or(not_found!("BlockId")))
    }

    fn blocks_ids(
        &self,
        start: Option<BlockHeight>,
        direction: IterDirection,
    ) -> BoxedIter<'_, StorageResult<(BlockHeight, BlockId)>> {
        self.all_block_ids(start, direction)
            .map(|result| result.map_err(StorageError::from))
            .into_boxed()
    }

    fn ids_of_latest_block(&self) -> StorageResult<(BlockHeight, BlockId)> {
        Ok(self
            .ids_of_latest_block()
            .transpose()
            .ok_or(not_found!("BlockId"))??)
    }
}

impl DatabaseTransactions for Database {
    fn tx_status(&self, tx_id: &TxId) -> StorageResult<TransactionStatus> {
        Ok(self
            .get_tx_status(tx_id)
            .transpose()
            .ok_or(not_found!("TransactionId"))??)
    }

    fn owned_transactions_ids(
        &self,
        owner: &Address,
        start: Option<TxPointer>,
        direction: IterDirection,
    ) -> BoxedIter<StorageResult<(TxPointer, TxId)>> {
        let start = start.map(|tx_pointer| OwnedTransactionIndexCursor {
            block_height: tx_pointer.block_height().into(),
            tx_idx: tx_pointer.tx_index(),
        });
        self.owned_transactions(owner, start, Some(direction))
            .map(|result| result.map_err(StorageError::from))
            .into_boxed()
    }
}

impl DatabaseMessages for Database {
    fn owned_message_ids(
        &self,
        owner: &Address,
        start_message_id: Option<MessageId>,
        direction: IterDirection,
    ) -> BoxedIter<'_, StorageResult<MessageId>> {
        self.owned_message_ids(owner, start_message_id, Some(direction))
            .map(|result| result.map_err(StorageError::from))
            .into_boxed()
    }

    fn all_messages(
        &self,
        start_message_id: Option<MessageId>,
        direction: IterDirection,
    ) -> BoxedIter<'_, StorageResult<Message>> {
        self.all_messages(start_message_id, Some(direction))
            .map(|result| result.map_err(StorageError::from))
            .into_boxed()
    }
}

impl DatabaseCoins for Database {
    fn owned_coins_ids(
        &self,
        owner: &Address,
        start_coin: Option<UtxoId>,
        direction: IterDirection,
    ) -> BoxedIter<'_, StorageResult<UtxoId>> {
        self.owned_coins_ids(owner, start_coin, Some(direction))
            .map(|res| res.map_err(StorageError::from))
            .into_boxed()
    }
}

impl DatabaseContracts for Database {
    fn contract_balances(
        &self,
        contract: ContractId,
        start_asset: Option<AssetId>,
        direction: IterDirection,
    ) -> BoxedIter<StorageResult<ContractBalance>> {
        self.contract_balances(contract, start_asset, Some(direction))
            .map(move |result| {
                result
                    .map_err(StorageError::from)
                    .map(|(asset_id, amount)| ContractBalance {
                        owner: contract,
                        amount,
                        asset_id,
                    })
            })
            .into_boxed()
    }
}

impl DatabaseChain for Database {
    fn chain_name(&self) -> StorageResult<String> {
        pub const DEFAULT_NAME: &str = "Fuel.testnet";

        Ok(self
            .get_chain_name()?
            .unwrap_or_else(|| DEFAULT_NAME.to_string()))
    }

    fn base_chain_height(&self) -> StorageResult<DaBlockHeight> {
        #[cfg(feature = "relayer")]
        {
            use fuel_core_relayer::ports::RelayerDb;
            self.get_finalized_da_height()
        }
        #[cfg(not(feature = "relayer"))]
        {
            Ok(0u64.into())
        }
    }
}

impl DatabasePort for Database {}