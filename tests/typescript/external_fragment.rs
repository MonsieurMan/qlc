use crate::helpers::{assert_generated, qlc_command_with_fake_dir_and_schema};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;

#[test]
fn compile_single_fragment() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    ...testFragment
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            "
fragment testFragment on Viewer {
  user { id }
}
        ",
        )
        .unwrap();

    cmd.assert().success();
    let temp_dir = temp_dir.persist_if(true);
    assert_generated(
        &temp_dir,
        "testQuery.ts",
        "
export interface TestQuery_viewer_user {
  id: string;
}

export interface TestQuery_viewer {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
}

export interface TestQuery {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
}
    ",
    );
}

#[test]
fn compile_recurisve_fragment_with_global() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    ...testFragment
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            r#"
#import "./userFragmentOne.graphql"
#import "./test/userFragmentTwo.graphql"

fragment testFragment on Viewer {
  user {
    id
    roles
    ...userFragmentOne
    ...userFragmentTwo
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("userFragmentOne.graphql")
        .write_str(
            "
fragment userFragmentOne on User {
  systemId: system_id
}
        ",
        )
        .unwrap();

    temp_dir
        .child("test")
        .child("userFragmentTwo.graphql")
        .write_str(
            "
fragment userFragmentTwo on User {
  featureList: feature_list
  singleUse: single_use
  scheduled_tiers {
    active
    endAt: end_at
  }
}
        ",
        )
        .unwrap();

    cmd.assert().success();
    let temp_dir = temp_dir.persist_if(true);
    assert_generated(
        &temp_dir,
        "testQuery.ts",
        r#"
import { UserRole, Feature } from "__generated__/globalTypes";

export interface TestQuery_viewer_user_scheduled_tiers {
  /**
   * Flag indicating if currently running and active schedule
   */
  active: boolean;
  /**
   * Tier schedule end time or indefinite
   */
  endAt: any | null;
}

export interface TestQuery_viewer_user {
  id: string;
  roles: (UserRole | null)[] | null;
  systemId: number | null;
  /**
   * An user's active features and features inherited from tier
   */
  featureList: (Feature)[];
  /**
   * Whether or not user is being used for single transaction
   */
  singleUse: boolean;
  /**
   * A user's scheduled tiers including historical, current and future
   */
  scheduled_tiers: (TestQuery_viewer_user_scheduled_tiers)[];
}

export interface TestQuery_viewer {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
}

export interface TestQuery {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
}
    "#,
    );
}