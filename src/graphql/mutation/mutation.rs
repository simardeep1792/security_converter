use async_graphql::*;

// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

use crate::graphql::{mutation::{ConversionRequestMutation, UserMutation}};

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    ConversionRequestMutation, 
/*
PersonMutation,
RoleMutation,
CapabilityMutation,
SkillMutation,
 */
);