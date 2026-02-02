use async_graphql::*;

// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

use crate::graphql::mutation::{AuthorityMutation, ConversionRequestMutation, UserMutation};

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    AuthorityMutation,
    ConversionRequestMutation,
);