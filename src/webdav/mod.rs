use icalendar::Calendar as ICalendar;
use icalendar::parser::{read_calendar as read_icalendar, unfold};
use reqwest::Error as ReqwestError;

use crate::webdav::calendar::Calendar;
use crate::webdav::calendar::vevent::VEvent;
use crate::webdav::calendar::vtimezone::VTimezone;
use crate::webdav::calendar::vtodo::VTodo;
use crate::webdav::connection::Connection;

pub mod parsing;
pub mod response;
mod connection;
pub mod calendar;

/*
pub(crate) fn read_calendar(conf: &str, calendar_name: &str) -> Result<Calendar,ReqwestError> {
    let result: Result<Connection,String> = Connection::new(&format!("data/{}",conf));
    let connection: Connection = result.unwrap();

    let mut calendar: Calendar = Calendar::new();
    let mut events: Vec<VEvent> = Vec::new();
    let responses: Vec<response::Response> = connection.get_responses(&format!("//{}",calendar_name)).unwrap();
    for response in responses {

        if response.href.ends_with("ics") {
            let event_string: Result<String, ReqwestError> = connection.get_event(&format!("//{}/{}", calendar_name, response.ical_file));

            match event_string {
                Ok(event_string) => {
                    events.push(parse_ical_object(&event_string))
                },
                Err(e) => return Err(e)
            }
        } else {
            calendar.name = response.prop.displayname;
            calendar.timezone = parse_ical_object(&response.prop.calendar_timezone);
        }
    }
    calendar.events = events;
    Ok(calendar)
}
 */

pub(crate) fn read_calendar(conf: &str, calendar_name: &str) -> Option<Calendar>{
    // connection
    let result: Result<Connection,String> = Connection::new(&format!("data/{}",conf));
    let connection: Connection = result.unwrap();

    let mut icals: Vec<ICalendar> = Vec::new();

    // iterate responses from xml
    let responses: Vec<response::Response> = connection.get_responses(&format!("//{}",calendar_name)).unwrap();
    for response in responses {

        // response links an ics file
        if response.href.ends_with("ics") {
            let ics_string: Result<String, ReqwestError> = connection.get_ics_file(&format!("//{}/{}", calendar_name, response.ical_file));

            match ics_string {
                Ok(ics_string) => {
                    let unfolded = unfold(&ics_string);
                    let result = read_icalendar(&unfolded);

                    if result.is_ok() {
                        // ICalender has two calendar classes. A parser and an actual class.
                        // into() turns the parser class into the actual one.
                        icals.push(result.unwrap().into());
                    }

                }
                Err(_) => ()
            }
        }
    }

    // put icals in event or todo list
    let mut events: Vec<VEvent> = Vec::new();
    let mut todos: Vec<VTodo> = Vec::new();

    for ical in icals {
        let event = VEvent::new(&ical);
        match event {
            Some(event) => events.push(event),
            None => ()
        }

        let todo = VTodo::new(&ical);
        match todo {
            Some(todo) => todos.push(todo),
            None => ()
        }
    }

    Some(
        Calendar{
            name: calendar_name.to_string(),
            events,
            todos,
            timezone: VTimezone::new()
        }
    )
}