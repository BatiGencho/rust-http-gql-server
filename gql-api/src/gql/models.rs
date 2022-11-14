use super::error::GqlError;
use crate::db::models::{DbEvent, DbTicket, DbUser};
use chrono::NaiveDateTime;
use juniper::GraphQLEnum;
use serde::{Deserialize, Serialize};
use std::{convert::From, fmt};

//--------------------------NFTS---------------------------------

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Gql request type for minting nft tickets")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMintNftsRequest {
    #[graphql(description = "Ticket id to mint tickets for")]
    pub ticket_id: String,
}

#[derive(juniper::GraphQLObject)]
#[graphql(description = "Gql response type for minting nfts")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMintNftsResponse {
    #[graphql(description = "Tx hash")]
    pub tx_hash: String,
}

//--------------------------USERS---------------------------------

#[derive(juniper::GraphQLObject)]
#[graphql(description = "Gql type for an existing user")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[graphql(description = "The user's id")]
    pub id: String,
    #[graphql(description = "The user's name")]
    pub name: Option<String>,
    #[graphql(description = "The user's username")]
    pub username: String,
    #[graphql(description = "The user's email")]
    pub email: Option<String>,
    #[graphql(description = "The user's phone number")]
    pub phone_number: Option<String>,
    #[graphql(description = "The user's registration date")]
    pub created_at: NaiveDateTime,
    #[graphql(description = "The user's wallet id")]
    pub wallet_id: String,
    #[graphql(description = "The user's wallet balance")]
    pub wallet_balance: String,
    #[graphql(description = "The user's type")]
    pub user_type: String,
    #[graphql(description = "The users's status")]
    pub user_status: String,
}

impl From<DbUser> for User {
    fn from(user: DbUser) -> Self {
        User {
            id: user.id.to_string(),
            name: user.name,
            username: user.username,
            email: user.email,
            phone_number: user.phone_number,
            created_at: user.created_at,
            wallet_id: user.wallet_id,
            wallet_balance: user.wallet_balance,
            user_type: user.user_type.to_string(),
            user_status: user.user_status.to_string(),
        }
    }
}

//--------------------------EVENTS---------------------------------

#[derive(juniper::GraphQLObject)]
#[graphql(description = "Gql type for an existing event")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[graphql(description = "The event's id")]
    pub id: String,
    #[graphql(description = "The event's name")]
    pub event_name: String,
    #[graphql(description = "The event's slug")]
    pub event_slug: String,
    #[graphql(description = "The event's starting date")]
    pub start_date: Option<NaiveDateTime>,
    #[graphql(description = "The event's end date")]
    pub end_date: Option<NaiveDateTime>,
    #[graphql(description = "The event's entry time")]
    pub entry_time: Option<NaiveDateTime>,
    #[graphql(description = "The event's timestamp")]
    pub created_at: NaiveDateTime,
    #[graphql(description = "The event's description")]
    pub description: Option<String>,
    #[graphql(description = "The event's virtual trait")]
    pub is_virtual: Option<bool>,
    #[graphql(description = "The event's featured trait")]
    pub is_featured: Option<bool>,
    #[graphql(description = "The event's venue name")]
    pub venue_name: Option<String>,
    #[graphql(description = "The event's venue location")]
    pub venue_location: Option<String>,
    #[graphql(description = "The event's cover photo url")]
    pub cover_photo_url: Option<String>,
    #[graphql(description = "The event's thumbnail url")]
    pub thumbnail_url: Option<String>,
    #[graphql(description = "The event's status")]
    pub event_status: String,
    #[graphql(description = "The event's creator id")]
    pub created_by_user: String,
    #[graphql(description = "The event's tickets")]
    pub tickets: Vec<Ticket>,
}

impl Event {
    pub fn new(event: DbEvent, tickets: Vec<DbTicket>) -> Self {
        Event {
            id: event.id.to_string(),
            event_name: event.event_name,
            event_slug: event.event_slug,
            start_date: event.start_date,
            end_date: event.end_date,
            entry_time: event.entry_time,
            created_at: event.created_at,
            description: event.description,
            is_virtual: event.is_virtual,
            is_featured: event.is_featured,
            venue_name: event.venue_name,
            venue_location: event.venue_location,
            cover_photo_url: event.cover_photo_url,
            thumbnail_url: event.thumbnail_url,
            event_status: event.event_status.to_string(),
            created_by_user: event.created_by_user.to_string(),
            tickets: tickets.into_iter().map(Ticket::from).collect(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Gql type for a new event")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEvent {
    #[graphql(description = "New event's name")]
    pub event_name: String,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Gql type for an update event")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEvent {
    #[graphql(description = "The event's id")]
    pub id: String,
    #[graphql(description = "The event's name")]
    pub event_name: Option<String>,
    #[graphql(description = "The event's starting date")]
    pub start_date: Option<NaiveDateTime>,
    #[graphql(description = "The event's end date")]
    pub end_date: Option<NaiveDateTime>,
    #[graphql(description = "The event's entry time")]
    pub entry_time: Option<NaiveDateTime>,
    #[graphql(description = "The event's description")]
    pub description: Option<String>,
    #[graphql(description = "The event's virtual trait")]
    pub is_virtual: Option<bool>,
    #[graphql(description = "The event's featured trait")]
    pub is_featured: Option<bool>,
    #[graphql(description = "The event's venue name")]
    pub venue_name: Option<String>,
    #[graphql(description = "The event's venue location")]
    pub venue_location: Option<String>,
    #[graphql(description = "The event's cover photo (base64)")]
    pub cover_photo_base64: Option<String>,
    #[graphql(description = "The event's thumbnail (base64)")]
    pub thumbnail_base64: Option<String>,
}

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventFilter {
    #[graphql(name = "FEATURED")]
    Featured,
    #[graphql(name = "NONE_FEATURED")]
    NoneFeatured,
    #[graphql(name = "ALL")]
    All,
}

/// Event Status
#[repr(i16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, GraphQLEnum)]
pub enum EventStatus {
    #[graphql(name = "DRAFT")]
    Draft = 0,
    #[graphql(name = "MINTING")]
    Minting = 1,
    #[graphql(name = "FINAL")]
    Final = 2,
}

impl From<EventStatus> for i16 {
    fn from(status: EventStatus) -> i16 {
        status as i16
    }
}

impl TryFrom<i16> for EventStatus {
    type Error = GqlError;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(EventStatus::Draft),
            1 => Ok(EventStatus::Minting),
            2 => Ok(EventStatus::Final),
            _ => Err(GqlError::UnknownEventStatus(n.to_string())),
        }
    }
}

/// Maps a string to a EventStatus
impl EventStatus {
    pub fn from_str(status: &str) -> EventStatus {
        let status = status.to_lowercase();
        match status.as_str() {
            "draft" => EventStatus::Draft,
            "minting" => EventStatus::Minting,
            "final" => EventStatus::Final,
            _ => EventStatus::Draft,
        }
    }
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStatus::Draft => write!(f, "draft"),
            EventStatus::Minting => write!(f, "minting"),
            EventStatus::Final => write!(f, "final"),
        }
    }
}
//-------------------------------TICKETS---------------------------------------//
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Gql type for an existing event ticket")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    #[graphql(description = "The ticket's id")]
    pub id: String,
    #[graphql(description = "The ticket's creation date")]
    pub created_at: NaiveDateTime,
    #[graphql(description = "The ticket's name")]
    pub ticket_name: String,
    #[graphql(description = "The tickets's slug")]
    pub ticket_slug: String,
    #[graphql(description = "The tickets's description")]
    pub description: Option<String>,
    #[graphql(description = "The tickets's price")]
    pub price: Option<String>,
    #[graphql(description = "The tickets's max release price")]
    pub max_release_price: Option<String>,
    #[graphql(description = "The ticket's available quantity")]
    pub quantity_available: Option<i32>,
    #[graphql(description = "The ticket's minimum purchase quantity")]
    pub min_purchase_quantity: Option<i32>,
    #[graphql(description = "The ticket's maximum purchase quantity")]
    pub max_purchase_quantity: Option<i32>,
    #[graphql(description = "Are transfers for that ticket allowed?")]
    pub allow_transfers: Option<bool>,
    #[graphql(description = "The ticket's associated event id")]
    pub event_id: String,
}

impl From<DbTicket> for Ticket {
    fn from(ticket: DbTicket) -> Self {
        Ticket {
            id: ticket.id.to_string(),
            created_at: ticket.created_at,
            ticket_name: ticket.ticket_name,
            ticket_slug: ticket.ticket_slug,
            description: ticket.description,
            price: ticket.price,
            max_release_price: ticket.max_release_price,
            quantity_available: ticket.quantity_available,
            min_purchase_quantity: ticket.min_purchase_quantity,
            max_purchase_quantity: ticket.max_purchase_quantity,
            allow_transfers: ticket.allow_transfers,
            event_id: ticket.event_id.to_string(),
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Gql type for creating a new event ticket")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTicket {
    #[graphql(description = "The ticket's name")]
    pub ticket_name: String,
    #[graphql(description = "The tickets's description")]
    pub description: Option<String>,
    #[graphql(description = "The tickets's price")]
    pub price: Option<String>,
    #[graphql(description = "The tickets's max release price")]
    pub max_release_price: Option<String>,
    #[graphql(description = "The ticket's available quantity")]
    pub quantity_available: Option<i32>,
    #[graphql(description = "The ticket's minimum purchase quantity")]
    pub min_purchase_quantity: Option<i32>,
    #[graphql(description = "The ticket's maximum purchase quantity")]
    pub max_purchase_quantity: Option<i32>,
    #[graphql(description = "Are transfers for that ticket allowed?")]
    pub allow_transfers: Option<bool>,
    #[graphql(description = "The ticket's associated event id")]
    pub event_id: String,
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Gql type for updating an existing event ticket")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTicket {
    #[graphql(description = "The ticket's id")]
    pub id: String,
    #[graphql(description = "The ticket's name")]
    pub ticket_name: Option<String>,
    #[graphql(description = "The tickets's description")]
    pub description: Option<String>,
    #[graphql(description = "The tickets's price")]
    pub price: Option<String>,
    #[graphql(description = "The tickets's max release price")]
    pub max_release_price: Option<String>,
    #[graphql(description = "The ticket's available quantity")]
    pub quantity_available: Option<i32>,
    #[graphql(description = "The ticket's minimum purchase quantity")]
    pub min_purchase_quantity: Option<i32>,
    #[graphql(description = "The ticket's maximum purchase quantity")]
    pub max_purchase_quantity: Option<i32>,
    #[graphql(description = "Are transfers for that ticket allowed?")]
    pub allow_transfers: Option<bool>,
}
