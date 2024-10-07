use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(integer(Transaction::Id))
                    .col(string_uniq(Transaction::TransactionHash))
                    .col(string(Transaction::SignerAccountId))
                    .col(string(Transaction::ReceiverAccountId))
                    .col(string(Transaction::BlockTimestamp))
                    .col(integer(Transaction::BlockHeight))
                    .col(string(Transaction::Action))
                    .col(string(Transaction::Method))
                    .col(boolean(Transaction::OutcomesStatus))
                    .col(string(Transaction::TweetID))
                    .col(string(Transaction::ImageURL))
                    .col(string_null(Transaction::UserToNotify))
                    .col(string_null(Transaction::NotarizedProof))
                    .col(string_null(Transaction::ZkProof))
                    .primary_key(
                        Index::create()
                            .name("pk-transaction")
                            .col(Transaction::Id)
                    )
                    .to_owned(),
            )
            .await?;

            manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-transaction-hash")
                    .table(Transaction::Table)
                    .col(Transaction::TransactionHash).to_owned()                 
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.drop_index(Index::drop().name("idx-transaction-hash").to_owned()   )
        .await?;

        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transaction {
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
    UserToNotify,
    NotarizedProof,
    ZkProof
}
