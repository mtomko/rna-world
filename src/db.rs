use crate::{dna::RestrictionEnzyme, errors::RWError};
use deadpool_postgres::Client;
use std::time::SystemTime;
use tokio_pg_mapper::FromTokioPostgresRow;
use uuid::Uuid;

pub async fn restriction_enzymes(client: &Client) -> Result<Vec<RestrictionEnzyme>, RWError> {
    let stmt = include_str!("../sql/list_restriction_enzymes.sql");
    let stmt = stmt.replace("$table_fields", &RestrictionEnzyme::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    client
        .query(&stmt, &[])
        .await
        .map(|rows| {
            rows.iter()
                .map(|row| RestrictionEnzyme::from_row_ref(row).unwrap())
                .collect::<Vec<RestrictionEnzyme>>()
        })
        .map_err(RWError::PGError)
}

pub async fn add_restriction_enzyme(
    client: &Client,
    restriction_enzyme: &RestrictionEnzyme,
) -> Result<(), RWError> {
    let stmt = include_str!("../sql/add_restriction_enzyme.sql");
    let stmt = stmt.replace("$table_fields", &RestrictionEnzyme::sql_table_fields());
    let stmt = client.prepare(&stmt).await.unwrap();

    client
        .query(
            &stmt,
            &[
                &Uuid::new_v4(),
                &restriction_enzyme.name,
                &restriction_enzyme.recognition_sequence,
                &SystemTime::now(),
            ],
        )
        .await
        .map(|_| ())
        .map_err(RWError::PGError)
}
