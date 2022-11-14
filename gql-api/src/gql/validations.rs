use super::{error::GqlError, models::UpdateEvent};
use crate::{
    db::models::{DbEvent, DbTicket},
    gql::{
        error::ValidationError,
        models::{NewTicket, UpdateTicket},
    },
};
use slugify::slugify;

pub fn update_event_mutation_payload<'a>(
    update_event: UpdateEvent,
    db_event: &'a mut DbEvent,
) -> Result<&'a mut DbEvent, GqlError> {
    // check event name
    if update_event
        .event_name
        .as_ref()
        .and_then(|f| Some(f.is_empty() || f.len() > 20))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_name",
            "Event name does not cover length requirements (max 20 chars)",
        )));
    }

    // check start date
    if update_event
        .start_date
        .as_ref()
        .and_then(|date| Some(date.timestamp_millis() < db_event.created_at.timestamp_millis()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_start_date",
            "Event start date lies behind the event creation date",
        )));
    }

    // check end date
    if update_event
        .end_date
        .as_ref()
        .and_then(|date| Some(date.timestamp_millis() < db_event.created_at.timestamp_millis()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_end_date",
            "Event end date lies behind the event creation date",
        )));
    }

    // check entry timedate
    if update_event
        .entry_time
        .as_ref()
        .and_then(|date| Some(date.timestamp_millis() < db_event.created_at.timestamp_millis()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_entry_date",
            "Event entry date lies behind the event creation date",
        )));
    }

    let start_date = update_event.start_date.clone().or(db_event.start_date);
    let end_date = update_event.end_date.clone().or(db_event.end_date);
    let entry_time = update_event.entry_time.clone().or(db_event.entry_time);

    // check start_date < end_date
    match (start_date.as_ref(), end_date.as_ref()) {
        (Some(start_date), Some(end_date)) => {
            if start_date.timestamp_millis() >= end_date.timestamp_millis() {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_start_end_date",
                    "Event end date must be after the event start date",
                )));
            }
        }
        _ => (),
    }

    // check entry_time < end_date
    match (entry_time.as_ref(), end_date.as_ref()) {
        (Some(entry_time), Some(end_date)) => {
            if entry_time.timestamp_millis() >= end_date.timestamp_millis() {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_end_entrytime_date",
                    "Event end date must be after the event entry time",
                )));
            }
        }
        _ => (),
    }

    // check entry_time > start_date
    match (entry_time.as_ref(), start_date.as_ref()) {
        (Some(entry_time), Some(start_date)) => {
            if entry_time.timestamp_millis() <= start_date.timestamp_millis() {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_start_entrytime_date",
                    "Event start date must be before the event entry time",
                )));
            }
        }
        _ => (),
    }

    // check description
    if update_event
        .description
        .as_ref()
        .and_then(|f| Some(f.is_empty() || f.len() > 20))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_description",
            "Event description does not cover length requirements (max 20 chars)",
        )));
    }

    // check venue name
    if update_event
        .venue_name
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_venue_name",
            "Event venue_name does not cover length requirements (should not be empty)",
        )));
    }

    // check venue location
    if update_event
        .venue_location
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_venue_location",
            "Event venue_location does not cover length requirements (should not be empty)",
        )));
    }

    // check cover photo url
    if update_event
        .cover_photo_base64
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_cover_photo",
            "Cover photo does not cover length requirements (should not be empty)",
        )));
    }

    // check thumbnail url
    if update_event
        .thumbnail_base64
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "event_thumbnail",
            "Event thumbnail url does not cover length requirements (should not be empty)",
        )));
    }

    // update the current db record
    if let Some(event_name) = update_event.event_name.as_ref() {
        db_event.event_name = event_name.to_string();
        db_event.event_slug = slugify!(event_name, separator = "-");
    }
    if update_event.start_date.is_some() {
        db_event.start_date = update_event.start_date;
    }
    if update_event.end_date.is_some() {
        db_event.end_date = update_event.end_date;
    }
    if update_event.entry_time.is_some() {
        db_event.entry_time = update_event.entry_time;
    }
    if update_event.description.is_some() {
        db_event.description = update_event.description;
    }
    if update_event.is_virtual.is_some() {
        db_event.is_virtual = update_event.is_virtual;
    }
    if update_event.is_featured.is_some() {
        db_event.is_featured = update_event.is_featured;
    }
    if update_event.venue_name.is_some() {
        db_event.venue_name = update_event.venue_name;
    }
    if update_event.venue_location.is_some() {
        db_event.venue_location = update_event.venue_location;
    }

    Ok(db_event)
}

pub fn check_new_ticket_payload(new_ticket: &NewTicket) -> Result<(), GqlError> {
    // check ticket name
    if new_ticket.ticket_name.len() > 20 {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_name",
            "Ticket name does not cover length requirements (max 20 chars)",
        )));
    }

    // check ticket description
    if new_ticket
        .description
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_description",
            "Ticket description does not cover length requirements (should not be empty)",
        )));
    }

    // check quantity available
    if new_ticket
        .quantity_available
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_quantity_available",
            "Ticket quantity does not cover requirements (should not be zero)",
        )));
    }

    // check min purchase quantity
    if new_ticket
        .min_purchase_quantity
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_min_purchase_quantity",
            "Ticket minimum purchase quantity does not cover requirements (should not be zero)",
        )));
    }

    // check max purchase quantity
    if new_ticket
        .max_purchase_quantity
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_max_purchase_quantity",
            "Ticket maximum purchase quantity does not cover requirements (should not be zero)",
        )));
    }

    // check min_purchase_quantity < max_purchase_quantity
    match (
        new_ticket.min_purchase_quantity.as_ref(),
        new_ticket.max_purchase_quantity.as_ref(),
    ) {
        (Some(min_purchase_quantity), Some(max_purchase_quantity)) => {
            if min_purchase_quantity > max_purchase_quantity {
                return Err(GqlError::Validation(ValidationError::new(
                    "ticket_min_max_purchase_quantity",
                    "Ticket min. purchase quantity must be less than the maximum",
                )));
            }
        }
        _ => (),
    }

    // check ticket price
    if new_ticket
        .price
        .as_ref()
        .map(|f| f.parse::<f64>())
        .transpose()
        .is_err()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_price",
            "Ticket price is unparsable",
        )));
    }

    // check ticket max release price
    if new_ticket
        .max_release_price
        .as_ref()
        .map(|f| f.parse::<f64>())
        .transpose()
        .is_err()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_max_release_price",
            "Ticket max. release price is unparsable",
        )));
    }

    Ok(())
}

pub fn update_ticket_mutation_payload<'a>(
    update_ticket: UpdateTicket,
    db_event: &DbEvent,
    db_ticket: &'a mut DbTicket,
) -> Result<&'a mut DbTicket, GqlError> {
    // check ticket name
    if update_ticket
        .ticket_name
        .as_ref()
        .and_then(|f| Some(f.is_empty() || f.len() > 20))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_name",
            "Ticket name does not cover length requirements (max 20 chars)",
        )));
    }

    // check ticket description
    if update_ticket
        .description
        .as_ref()
        .and_then(|f| Some(f.is_empty()))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_description",
            "Ticket description does not cover length requirements (should not be empty)",
        )));
    }

    // check quantity available
    if update_ticket
        .quantity_available
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_quantity_available",
            "Ticket quantity does not cover requirements (should not be zero)",
        )));
    }

    // check min purchase quantity
    if update_ticket
        .min_purchase_quantity
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_min_purchase_quantity",
            "Ticket minimum purchase quantity does not cover requirements (should not be zero)",
        )));
    }

    // check max purchase quantity
    if update_ticket
        .max_purchase_quantity
        .as_ref()
        .and_then(|f| Some(f == &0))
        .unwrap_or_default()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_max_purchase_quantity",
            "Ticket maximum purchase quantity does not cover requirements (should not be zero)",
        )));
    }

    // check min_purchase_quantity < max_purchase_quantity
    match (
        update_ticket.min_purchase_quantity.as_ref(),
        update_ticket.max_purchase_quantity.as_ref(),
    ) {
        (Some(min_purchase_quantity), Some(max_purchase_quantity)) => {
            if min_purchase_quantity > max_purchase_quantity {
                return Err(GqlError::Validation(ValidationError::new(
                    "ticket_min_max_purchase_quantity",
                    "Ticket min. purchase quantity must be less than the maximum",
                )));
            }
        }
        _ => (),
    }

    // check ticket price
    if update_ticket
        .price
        .as_ref()
        .map(|f| f.parse::<f64>())
        .transpose()
        .is_err()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_price",
            "Ticket price is unparsable",
        )));
    }

    // check ticket max release price
    if update_ticket
        .max_release_price
        .as_ref()
        .map(|f| f.parse::<f64>())
        .transpose()
        .is_err()
    {
        return Err(GqlError::Validation(ValidationError::new(
            "ticket_max_release_price",
            "Ticket max. release price is unparsable",
        )));
    }

    // update the current db record
    if let Some(ticket_name) = update_ticket.ticket_name.as_ref() {
        let ticket_slug = format!(
            "{}-{}",
            &db_event.event_slug,
            slugify!(&ticket_name, separator = "-")
        );
        db_ticket.ticket_name = ticket_name.to_string();
        db_ticket.ticket_slug = ticket_slug;
    }
    if update_ticket.description.is_some() {
        db_ticket.description = update_ticket.description;
    }
    if update_ticket.price.is_some() {
        db_ticket.price = update_ticket.price;
    }
    if update_ticket.max_release_price.is_some() {
        db_ticket.max_release_price = update_ticket.max_release_price;
    }
    if update_ticket.quantity_available.is_some() {
        db_ticket.quantity_available = update_ticket.quantity_available;
    }
    if update_ticket.min_purchase_quantity.is_some() {
        db_ticket.min_purchase_quantity = update_ticket.min_purchase_quantity;
    }
    if update_ticket.max_purchase_quantity.is_some() {
        db_ticket.max_purchase_quantity = update_ticket.max_purchase_quantity;
    }
    if update_ticket.allow_transfers.is_some() {
        db_ticket.allow_transfers = update_ticket.allow_transfers;
    }
    Ok(db_ticket)
}
