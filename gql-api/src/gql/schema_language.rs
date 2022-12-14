#![allow(missing_docs)]

/// The schema as a static/hardcoded GraphQL Schema Language.
pub const STATIC_GRAPHQL_SCHEMA_DEFINITION: &str = include_str!("../schemas/schema.graphql");

#[cfg(test)]
mod tests {
    use juniper::{EmptyMutation, EmptySubscription, RootNode};
    use pretty_assertions::assert_eq;

    use crate::{
        mutations::MutationRoot, quiries::QueryRoot,
        schema_language::STATIC_GRAPHQL_SCHEMA_DEFINITION, subscriptions::SubscriptionRoot,
    };

    #[test]
    fn dynamic_schema_language_matches_static() {
        let schema = RootNode::new(QueryRoot, MutationRoot, SubscriptionRoot);

        dbg!("{}", schema.as_schema_language());

        // `include_str()` keeps line endings. `git` will sadly by default
        // convert them, making this test fail without runtime tweaks on
        // Windows.
        //
        // See https://github.com/rust-lang/rust/pull/63681.
        #[cfg(windows)]
        let expected = &STATIC_GRAPHQL_SCHEMA_DEFINITION.replace("\r\n", "\n");

        #[cfg(not(windows))]
        let expected = STATIC_GRAPHQL_SCHEMA_DEFINITION;

        assert_eq!(expected, &schema.as_schema_language());
    }
}
