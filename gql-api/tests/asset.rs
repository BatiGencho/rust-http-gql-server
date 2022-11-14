use gql_api::db::models::AssetFile;

mod common;
use crate::common::gen_asset_file;

#[tokio::test]
async fn test_asset_files_crud() {
    let cfg = common::setup().await;

    let bucket = "some_bucket";
    let expected = gen_asset_file(bucket, cfg.event.id);
    let expected2 = gen_asset_file(bucket, cfg.event.id);
    let duplicated = AssetFile::new(
        bucket,
        expected2.s3_absolute_key.clone(),
        None,
        cfg.event.id,
    );

    gql_api::db::sql::insert_asset_file(&cfg.client, &expected)
        .await
        .expect("failed to insert s3 file");

    gql_api::db::sql::insert_asset_file(&cfg.client, &expected2)
        .await
        .expect("failed to insert s3 file (expected2)");

    gql_api::db::sql::insert_asset_file(&cfg.client, &duplicated)
        .await
        .expect_err("should not be possible to add a repeated file");

    let actual = gql_api::db::sql::db_get_asset_file(&cfg.client, &expected.id)
        .await
        .expect("unable to get expected file");
    assert_eq!(expected, actual);

    let mut actual_vec = gql_api::db::sql::db_get_files_for_event(&cfg.client, &expected.event_id)
        .await
        .expect("failed to fetch files for event");
    actual_vec.sort_by(|a, b| a.s3_absolute_key.cmp(&b.s3_absolute_key));
    let mut expected_vec = vec![expected, expected2];
    expected_vec.sort_by(|a, b| a.s3_absolute_key.cmp(&b.s3_absolute_key));

    assert_eq!(expected_vec, actual_vec);
}

#[tokio::test]
async fn test_asset_files_update() {
    let cfg = common::setup().await;

    let bucket = "some_bucket";
    let expected_ipfs_hash = uuid::Uuid::new_v4().to_string();
    let mut expected = gen_asset_file(bucket, cfg.event.id);

    gql_api::db::sql::insert_asset_file(&cfg.client, &expected)
        .await
        .expect("failed to insert s3 file");

    gql_api::db::sql::update_file_ipfs_hash(&cfg.client, &expected.id, &expected_ipfs_hash)
        .await
        .expect("failed to update ipfs hash");

    expected.ipfs_hash = Some(expected_ipfs_hash);

    let actual = gql_api::db::sql::db_get_asset_file(&cfg.client, &expected.id)
        .await
        .expect("unable to get expected file");
    assert_eq!(expected, actual);
}
