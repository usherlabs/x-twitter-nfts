use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NearTransaction::Table)
                    .if_not_exists()
                    .col(big_unsigned_uniq(NearTransaction::Id))
                    .col(string_uniq(NearTransaction::TransactionHash))
                    .col(string(NearTransaction::SignerAccountId))
                    .col(string(NearTransaction::ReceiverAccountId))
                    .col(string(NearTransaction::BlockTimestamp))
                    .col(integer(NearTransaction::BlockHeight))
                    .col(string(NearTransaction::Action))
                    .col(string(NearTransaction::Method))
                    .col(boolean(NearTransaction::OutcomesStatus))
                    .col(string(NearTransaction::TweetID))
                    .col(string(NearTransaction::ImageURL))
                    .col(string_null(NearTransaction::MintTransactionHash))
                    .col(string_null(NearTransaction::UserToNotify))
                    .col(string_null(NearTransaction::NotarizedProof))
                    .col(string_null(NearTransaction::ZkProof))
                    .primary_key(
                        Index::create()
                            .name("pk-transaction")
                            .col(NearTransaction::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-near-transaction-hash")
                    .table(NearTransaction::Table)
                    .col(NearTransaction::TransactionHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx-near-transaction-hash").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(NearTransaction::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NearTransaction {
    Table,
    Id,
    TransactionHash,
    SignerAccountId,
    ReceiverAccountId,
    BlockTimestamp,
    BlockHeight,
    Action,
    Method,
    OutcomesStatus,
    TweetID,
    ImageURL,
    MintTransactionHash,
    UserToNotify,
    NotarizedProof,
    ZkProof,
}
