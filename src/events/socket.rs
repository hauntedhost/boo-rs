use log::debug;

use crate::app::AppState;
use crate::socket::response::{parse_response, Response};

pub fn handle_message_event(app: &mut AppState, message_payload: String) {
    debug!("received message: {}", message_payload);
    app.append_log(message_payload.clone());

    match parse_response(&message_payload) {
        Response::Null => (),
        Response::JoinReply(reply) => {
            app.user.online_at = reply.user.online_at;
        }
        Response::Shout(shout) => {
            if !shout.user.uuid.eq(&app.user.uuid) {
                let message = format!("{}: {}", shout.user.username, shout.message);
                app.add_user_message(message);
            }
        }
        Response::RoomsUpdate(rooms) => {
            app.set_rooms(rooms);
        }
        Response::PresenceDiff(diff) => {
            for user in diff.joins {
                let message = format!("@{} has joined #{}", user.username, app.room);
                app.add_user(user);
                app.add_system_message(message);
            }
            for user in diff.leaves {
                let message = format!("{} has left {}", user.username, app.room);
                app.remove_user(user);
                app.add_system_message(message);
            }
        }
        Response::PresenceState(state) => {
            app.set_users(state.users);
        }
    }
}
